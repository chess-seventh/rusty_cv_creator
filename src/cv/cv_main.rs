use std::{fs::File, io::Write};

use latex::{Document, DocumentClass};

use crate::cv::{preambles::cv_preambles::build_preamble, sections::cv_sections::build_sections};

pub fn cv_generate() {
    let mut doc = Document::new(DocumentClass::Other("awesome-cv".to_string()));

    let preambles = build_preamble();

    for preamble in preambles {
        doc.preamble.push(preamble);
    }

    let sections_body = build_sections();

    for section in sections_body {
        doc.push(section);
    }

    let rendered = latex::print(&doc).unwrap();

    // println!("{rendered:}");

    let mut file = File::create(
        "/home/seventh/src/git.sr.ht/chess7th/rusty_cv_creator/latex_template/NewCVPiva.tex",
    )
    .expect("could not create file");
    file.write_all(rendered.as_bytes())
        .expect("could not write to file");
}

// \begin{document}
// \makecvheader
// \vspace{-5mm}
// \input{cv-sections/aboutme.tex}
// \vspace{-3mm}
// \input{cv-sections/skills.tex}
// % \vspace{-3mm}
// \input{cv-sections/fullexperience.tex}
// \vspace{-4mm}
// \input{cv-sections/education.tex}
// \vspace{-4mm}
// \input{cv-sections/extracurricular.tex}
// \vspace{-5mm}
// \blfootnote{\tiny{\bodyfontlight\upshape\color{gray}{Previous experiences available upon request.}}}
// \end{document}
