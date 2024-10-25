use latex::Element;

use crate::cv::helpers::{Experience, ExperienceType, LatexCvExperienceEntry};

pub fn piva_consulting(tasks_selector: Option<&ExperienceType>) -> Vec<Element> {
    LatexCvExperienceEntry::new(
        experience_piva_consulting_head(),
        experience_piva_consulting_body(tasks_selector).compile_elements(),
    )
    .compile_elements()
}

fn experience_piva_consulting_body(tasks_selector: Option<&ExperienceType>) -> Experience {
    match tasks_selector {
        Some(ExperienceType::DefaultExperience | &_) | None => Experience::Piva(piva_all()),
    }
}

fn piva_all() -> Vec<Element> {
    vec![piva_accomplishment_01(), piva_accomplishment_02()]
}

fn piva_accomplishment_01() -> Element {
    Element::UserDefined(
        r#"
        \item{🎯 Bridging the gap between development and operations teams, fostering seamless
        collaboration and communication.}
    "#
        .to_string(),
    )
}

fn piva_accomplishment_02() -> Element {
    Element::UserDefined(
        r#"
        \item{🎯 Implementing efficient workflows and advocating for best practices, driving
        enhanced software delivery processes, resulting in improved system reliability and
        accelerated deployment cycles.}
    "#
        .to_string(),
    )
}

fn experience_piva_consulting_head() -> Element {
    Element::UserDefined(
        r#"
        \cventry{🏢 Piva Consulting} % Organization
        {💻 Platform Engineer Consultant - Freelance DevOps \& Site Reliability Engineer} % Job title
        {Remote 📍} % Location
        {Mar. 2024 – Present 📆} % Date(s)
        {\begin{cvitems}
    "#
        .to_string(),
    )
}


// Element::UserDefined(
//     r#"
//             \item{🎯 Bridging the gap between development and operations teams, fostering seamless collaboration and communication.}
//             \item{🎯 Implementing efficient workflows and advocating for best practices, driving enhanced software delivery processes, resulting in improved system reliability and accelerated deployment cycles.}
// "#
//     .to_string(),
// )
