use rustc_hir::def_id::DefId;
use rustc_middle::ty::{
    subst::SubstsRef, AssocItemContainer::*, AssocKind, GenericPredicates, ParamEnv,
    ToPolyTraitRef, TraitPredicate, TraitRef, TyCtxt,
};
use rustc_trait_selection::traits::ImplSource;

use why3::declaration::{CloneKind, TyDecl, TyDeclKind};
use why3::{
    declaration::{CloneSubst, Decl, DeclClone, Module, ValKind::*},
    mlcfg::Type,
    QName,
};

use crate::{ctx, rustc_extensions};

use crate::ctx::*;
use crate::translation::ty::{self, translate_ty};
use crate::util::{ident_of, is_contract};

use rustc_resolve::Namespace;

impl<'tcx> TranslationCtx<'_, 'tcx> {
    // Translate a trait declaration
    pub fn translate_trait(&mut self, def_id: DefId) {
        if !self.translated_items.insert(def_id) {
            return;
        }

        let mut names = CloneMap::new(self.tcx, ItemType::Trait);
        names.clone_self(def_id);

        // The first predicate is a trait reference so we skip it
        for super_trait in traits_used_by(self.tcx, def_id).filter(|t| t.def_id() != def_id) {
            // Ensure trait depends on all super traits
            translate_constraint(self, &mut names, super_trait);
        }

        let mut decls: Vec<_> = Vec::new();
        decls.extend(own_generic_decls_for(self.tcx, def_id));
        decls.extend(names.to_clones(self));

        for item in self.tcx.associated_items(def_id).in_definition_order() {
            match item.kind {
                AssocKind::Fn => {
                    if is_contract(self.tcx, item.def_id) {
                        continue;
                    }

                    let mut sig = crate::util::signature_of(self, &mut names, item.def_id);

                    decls.extend(crate::translation::function::own_generic_decls_for(
                        self.tcx,
                        item.def_id,
                    ));

                    decls.extend(names.to_clones(self));
                    match crate::util::item_type(self.tcx, item.def_id) {
                        ItemType::Logic => decls.push(Decl::ValDecl(Function { sig })),
                        ItemType::Predicate => {
                            sig.retty = None;
                            decls.push(Decl::ValDecl(Predicate { sig }));
                        }
                        ItemType::Program => {
                            decls.push(Decl::ValDecl(Val { sig }));
                        }
                        _ => unreachable!(),
                    }
                }
                AssocKind::Type => {
                    let ty_name: why3::Ident = ty::ty_name(self.tcx, item.def_id).into();
                    decls.extend(names.to_clones(self));
                    decls.push(Decl::TyDecl(TyDecl {
                        ty_name,
                        ty_params: Vec::new(),
                        kind: TyDeclKind::Opaque,
                    }));
                }
                knd => unimplemented!("{:?} - {:?}", def_id, knd),
            }
        }

        let trait_name = translate_trait_name(self.tcx, def_id);

        self.add_trait(def_id, Module { name: trait_name.name(), decls });
    }

    pub fn translate_impl(&mut self, impl_id: DefId) {
        if !self.translated_items.insert(impl_id) {
            return;
        }

        let trait_ref = self.tcx.impl_trait_ref(impl_id).unwrap();
        let mut names = CloneMap::new(self.tcx, ItemType::Impl);

        self.translate_trait(trait_ref.def_id);

        let mut subst = ctx::base_subst(self, &mut names, trait_ref.def_id, trait_ref.substs);

        let mut assoc_types = Vec::new();
        for assoc in self.tcx.associated_items(impl_id).in_definition_order() {
            match assoc.kind {
                AssocKind::Fn => subst.extend(translate_assoc_function(self, &mut names, assoc)),
                AssocKind::Type => {
                    let assoc_ty = self.tcx.type_of(assoc.def_id);
                    // TODO: Clean up translation of names to handle this automatically
                    let name = ident_of(assoc.ident.name);
                    assoc_types.push(Decl::TyDecl(TyDecl {
                        ty_name: name.clone(),
                        ty_params: Vec::new(),
                        kind: TyDeclKind::Alias(ty::translate_ty(
                            self,
                            &mut names,
                            rustc_span::DUMMY_SP,
                            assoc_ty,
                        )),
                    }));

                    subst.push(CloneSubst::Type(
                        name.clone().into(),
                        Type::TConstructor(QName { module: vec![], name }),
                    ))
                }
                AssocKind::Const => self.crash_and_error(
                    self.tcx.span_of_impl(impl_id).unwrap_or(rustc_span::DUMMY_SP),
                    "Associated constants are not yet supported",
                ),
            }
        }

        let mut decls: Vec<_> = Vec::new();
        decls.extend(all_generic_decls_for(self.tcx, impl_id));
        decls.extend(names.to_clones(self));
        decls.extend(assoc_types);

        decls.push(Decl::Clone(DeclClone {
            name: translate_trait_name(self.tcx, trait_ref.def_id),
            subst,
            kind: CloneKind::Bare,
        }));

        let name = translate_value_id(self.tcx, impl_id);
        self.add_impl(impl_id, Module { name: name.name(), decls });
    }
}

pub fn translate_predicates(
    ctx: &mut TranslationCtx<'_, 'tcx>,
    names: &mut CloneMap<'tcx>,
    preds: GenericPredicates<'tcx>,
) {
    for (pred, _) in preds.predicates.iter() {
        use rustc_middle::ty::PredicateKind::*;
        match pred.kind().no_bound_vars().unwrap() {
            Trait(tp) => translate_constraint(ctx, names, tp),
            Projection(pp) => {
                let _ty = translate_ty(ctx, names, rustc_span::DUMMY_SP, pp.ty);
                names
                    .insert(pp.projection_ty.trait_def_id(ctx.tcx), pp.projection_ty.substs)
                    .add_projection((pp.projection_ty.item_def_id, pp.ty));
            }
            _ => continue,
        }
    }
}

pub fn traits_used_by(
    tcx: TyCtxt,
    def_id: DefId,
) -> impl Iterator<Item = TraitPredicate> {
    let predicates = tcx.predicates_of(def_id);

    predicates.predicates.iter().filter_map(|(pred, _)| {
        let inner = pred.kind().no_bound_vars().unwrap();
        use rustc_middle::ty::PredicateKind::*;
        match inner {
            Trait(tp) => Some(tp),
            _ => None,
        }
    })
}

pub fn translate_constraint<'tcx>(
    ctx: &mut TranslationCtx<'_, 'tcx>,
    names: &mut CloneMap<'tcx>,
    tp: TraitPredicate<'tcx>,
) {
    names.insert(tp.def_id(), tp.trait_ref.substs);

    // If we haven't seen this trait, first translate it
    ctx.translate_trait(tp.def_id());
}

use crate::function::{all_generic_decls_for, own_generic_decls_for};
use rustc_middle::ty::subst::InternalSubsts;
use rustc_middle::ty::AssocItem;

fn translate_assoc_function(
    ctx: &mut TranslationCtx<'_, 'tcx>,
    names: &'a mut CloneMap<'tcx>,
    assoc: &AssocItem,
) -> impl Iterator<Item = CloneSubst> + 'tcx {
    let impl_id = ctx.tcx.impl_of_method(assoc.def_id).unwrap();
    let trait_id = ctx.tcx.trait_id_of_impl(impl_id).unwrap();

    let assoc_subst = InternalSubsts::identity_for_item(ctx.tcx, impl_id);
    let name = names.insert(assoc.def_id, assoc_subst).clone();

    ctx.translate_function(assoc.def_id);

    let tcx = ctx.tcx;

    // Get the id of the generic version of the trait method
    let trait_method = tcx
        .associated_items(trait_id)
        .find_by_name_and_namespace(
            ctx.tcx,
            assoc.ident,
            Namespace::ValueNS, //TODO generalize this to include associated types
            trait_id,
        )
        .unwrap();

    // build the substitution between the concrete and generic versions
    let method_subst = tcx
        .generics_of(trait_method.def_id)
        .params
        .iter()
        .zip(tcx.generics_of(assoc.def_id).params.iter())
        .map(move |(tr_param, inst_param)| {
            CloneSubst::Type(
                (&*tr_param.name.as_str().to_lowercase()).into(),
                Type::TConstructor(name.qname(tcx, inst_param.def_id)),
            )
        });

    let assoc_method = if crate::util::is_predicate(ctx.tcx, assoc.def_id) {
        CloneSubst::Predicate(
            assoc.ident.to_string().into(),
            names.insert(assoc.def_id, assoc_subst).qname(ctx.tcx, assoc.def_id),
        )
    } else {
        CloneSubst::Val(
            assoc.ident.to_string().into(),
            names.insert(assoc.def_id, assoc_subst).qname(ctx.tcx, assoc.def_id),
        )
    };

    method_subst.chain(std::iter::once(assoc_method))
}

fn translate_trait_name(tcx: TyCtxt<'_>, def_id: DefId) -> QName {
    translate_value_id(tcx, def_id)
}

pub fn impl_or_trait(
    tcx: TyCtxt<'tcx>,
    param_env: ParamEnv<'tcx>,
    def_id: DefId,
    subst: SubstsRef<'tcx>,
) -> Option<(DefId, SubstsRef<'tcx>)> {
    if let Some(assoc) = tcx.opt_associated_item(def_id) {
        let target = match assoc.container {
            ImplContainer(_) => return Some((def_id, subst)),
            TraitContainer(def_id) => def_id,
        };
        let trait_ref = TraitRef { def_id: target, substs: subst }.to_poly_trait_ref();
        let source =
            rustc_extensions::codegen::codegen_fulfill_obligation(tcx, (param_env, trait_ref));

        if let Err(mut err) = source {
            err.cancel();
            return None;
        }
        match source.unwrap() {
            ImplSource::UserDefined(impl_data) => {
                let trait_def_id = tcx.trait_id_of_impl(impl_data.impl_def_id).unwrap();
                let trait_def = tcx.trait_def(trait_def_id);
                // Find the id of the actual associated method we will be running
                let leaf_def = trait_def
                    .ancestors(tcx, impl_data.impl_def_id)
                    .unwrap()
                    .leaf_def(tcx, assoc.ident, assoc.kind)
                    .unwrap_or_else(|| {
                        panic!("{:?} not found in {:?}", assoc, impl_data.impl_def_id);
                    });
                use rustc_trait_selection::infer::TyCtxtInferExt;

                // Translate the original substitution into one on the selected impl method
                let leaf_substs = tcx.infer_ctxt().enter(|infcx| {
                    let param_env = param_env.with_reveal_all_normalized(tcx);
                    let substs = subst.rebase_onto(tcx, trait_def_id, impl_data.substs);
                    let substs = rustc_trait_selection::traits::translate_substs(
                        &infcx,
                        param_env,
                        impl_data.impl_def_id,
                        substs,
                        leaf_def.defining_node,
                    );
                    infcx.tcx.erase_regions(substs)
                });

                Some((leaf_def.item.def_id, leaf_substs))
            }
            ImplSource::Param(_, _) => Some((def_id, subst)),
            _ => unimplemented!(),
        }
    } else {
        Some((def_id, subst))
    }
}
