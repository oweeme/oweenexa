//! Recoge los `src` estáticos de `<img>` en el IR de una página (Fase
//! 14, para el presupuesto `maxImage`) — mismo criterio que
//! `nexa_ui::collect_used_classes`: solo valores estáticos
//! (`src="/logo.png"`), nunca `src={expr}` (no hay forma de saber su
//! valor sin datos reales).

use nexa_ast::AttrValue;
use nexa_ir::{IrNode, IrNodeKind};

pub fn collect_image_srcs(root: &IrNode) -> Vec<String> {
    let mut out = Vec::new();
    visit(root, &mut out);
    out
}

fn visit(node: &IrNode, out: &mut Vec<String>) {
    let IrNodeKind::Element { tag, attrs, children, .. } = &node.kind else {
        return;
    };

    if tag == "img" {
        for attr in attrs {
            if attr.name == "src" {
                if let Some(AttrValue::Static(src)) = &attr.value {
                    out.push(src.clone());
                }
            }
        }
    }

    for child in children {
        visit(child, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_ast::Attr;
    use nexa_ir::Classification;

    fn element(tag: &str, attrs: Vec<Attr>, children: Vec<IrNode>) -> IrNode {
        IrNode { id: 0, classification: Classification::Static, kind: IrNodeKind::Element { tag: tag.into(), attrs, events: vec![], island: None, children } }
    }

    fn static_src(value: &str) -> Attr {
        Attr { name: "src".into(), value: Some(AttrValue::Static(value.into())) }
    }

    #[test]
    fn collects_static_image_srcs_at_any_depth() {
        let root = element(
            "main",
            vec![],
            vec![element("div", vec![], vec![element("img", vec![static_src("/logo.png")], vec![])])],
        );

        assert_eq!(collect_image_srcs(&root), vec!["/logo.png".to_string()]);
    }

    #[test]
    fn ignores_dynamic_image_srcs() {
        let root = element(
            "img",
            vec![Attr {
                name: "src".into(),
                value: Some(AttrValue::Dynamic(nexa_ast::Template::from_expr(nexa_ast::Expr::Identifier("x".into())))),
            }],
            vec![],
        );

        assert!(collect_image_srcs(&root).is_empty());
    }

    #[test]
    fn a_page_with_no_images_yields_nothing() {
        let root = element("main", vec![], vec![element("p", vec![], vec![])]);
        assert!(collect_image_srcs(&root).is_empty());
    }
}
