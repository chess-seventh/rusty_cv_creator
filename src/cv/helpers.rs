use latex::Element;

#[derive(Debug, Clone)]
pub struct LatexCvExperienceEntry {
    pub header: Element,
    pub body: Vec<Element>,
    footer: Element,
}

impl LatexCvExperienceEntry {
    pub fn new(header: Element, body: Vec<Element>) -> Self {
        Self {
            header,
            body,
            footer: experience_item_foot(),
        }
    }

    pub fn compile_elements(&self) -> Vec<Element> {
        let mut elements = vec![];
        elements.push(self.header.clone());
        elements.append(&mut self.body.clone());
        elements.push(self.footer.clone());
        elements
    }
}

pub fn experience_header() -> Element {
    Element::UserDefined(
        r#"
\cvsection{👔 Career Summary}{
    \begin{cventries}
    "#
        .to_string(),
    )
}

pub fn experience_footer() -> Element {
    Element::UserDefined(
        r#"
    \end{cventries}
}
    "#
        .to_string(),
    )
}

pub fn experience_item_foot() -> Element {
    Element::UserDefined(
        r#"
            \end{cvitems}
        }
        }

    "#
        .to_string(),
    )
}

pub fn separation_between_sections() -> Element {
    Element::UserDefined(
        r#"
                \\
                \newline
    "#
        .to_string(),
    )
}
// #[derive(Debug, Clone)]
pub enum ExperienceType {
    DefaultExperience, // contains all experiences in IT
    MiniExperience, // contains a shorter version of my experiences
    FullExperience, // contains complete with multiple entries per CV
    // CiCdExperience, // emphasis on the in CI/CD Experiences
    // AWSExperience, // emphasis on the AWS experiences
    // MonitoringExperience, // emphasis on the Monitoring & Observability
    FullHospitality, // contains default IT and all Hospitality experiences
}

#[derive(Debug, Clone)]
pub enum Experience {
    Ebu(Vec<Element>),
    Bestmile(Vec<Element>),
    OpenSystems(Vec<Element>),
    Zf(Vec<Element>),
    Piva(Vec<Element>),
    Pictet(Vec<Element>),
}

impl Experience {
    pub fn compile_elements(&self) -> Vec<Element> {
        match self {
            Experience::Ebu(elements) | Experience::Bestmile(elements) | Experience::OpenSystems(elements) | Experience::Zf(elements) | Experience::Piva(elements) | Experience::Pictet(elements) => elements.clone(),
        }
    }
}