use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

/// Template tree
#[derive(Default, Debug)]
pub struct Tree {
    pub root: PathBuf,
    pub items: Option<Vec<PathBuf>>,
    pub children: Option<Vec<Box<Tree>>>,
}
impl Tree {
    fn new(path: &Path) -> Self {
        Tree {
            root: path.to_path_buf(),
            ..Default::default()
        }
    }
    fn section(&self) -> &str {
        self.root.file_name().and_then(|i| i.to_str()).unwrap()
    }
    fn to_latex(&self) -> anyhow::Result<Vec<String>> {
        let mut inputs: Vec<String> = vec![];
        if let Some(children) = &self.children {
            for child in children {
                let mut doc = vec![format!(r#"\section{{{:}}}"#, child.section())];
                let mut file = File::create(child.root.join("section.tex"))?;
                doc.extend(child.to_latex()?);
                if let Some(items) = &child.items {
                    for item in items {
                        doc.push(format!(
                            r#"\subsubsection*{{{:}}}"#,
                            item.file_name().and_then(|i| i.to_str()).unwrap()
                        ));
                        doc.push(format!(
                            r#"\includegraphics[width=0.65\textwidth]{{{{{:?}}}}}"#,
                            item
                        ));
                    }
                };
                write!(file, r#"{:}"#, doc.join("\n"))?;
                inputs.push(format!(
                    r#"\input{{{:}}}"#,
                    child.root.join("section.tex").to_str().unwrap()
                ));
            }
        }
        Ok(inputs)
    }
}

/// Parse a 1-level down directory tree
fn visit_dirs(root: &Path, root_tree: &mut Tree) -> anyhow::Result<()> {
    for entry in fs::read_dir(root)? {
        let dir = entry?;
        let path = dir.path();
        if path.is_dir() {
            let mut tree = Tree::new(&path);
            visit_dirs(&path, &mut tree)?;
            root_tree
                .children
                .get_or_insert(vec![])
                .push(Box::new(tree))
        }
        if path.is_file()
            && (path.extension().unwrap() == "jpg" || path.extension().unwrap() == "png")
        {
            root_tree
                .items
                .get_or_insert(vec![])
                .push(path.with_extension(""));
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let argument: Vec<_> = env::args().skip(1).collect();
    if argument.len() != 1 {
        println!(r#"Usage: cargo run -- <full_path_to_repository>"#);
        panic!("Expected 1 argument found {}", argument.len())
    }

    let root = Path::new(&argument[0]);
    let template = include_str!("fem_validation.tex");
    let mut file = File::create(root.join("fem_validation.tex"))?;
    write!(file, r#"{:}"#, template)?;

    let mut tree = Tree::new(&root);
    visit_dirs(root, &mut tree)?;
    let inputs = tree.to_latex()?;
    let mut file = File::create(root.join("main.tex")).unwrap();
    write!(file, r#"{:}"#, inputs.join("\n"))?;
    Ok(())
}
