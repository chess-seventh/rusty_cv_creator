use crate::cv::helpers::{Experience, ExperienceType, LatexCvExperienceEntry};
use latex::Element;
use crate::cv::helpers::separation_between_sections;

pub fn open_systems(experience_keyword: Option<&ExperienceType>) -> Vec<Element> {
    LatexCvExperienceEntry::new(
        experience_os_head(),
        experience_os_body(experience_keyword).compile_elements()
    ).compile_elements()
}

// TODO: use match case to know which tasks or accomplishments I want to add.
fn experience_os_body(experience_type: Option<&ExperienceType>) -> Experience {
    match experience_type {
        Some(ExperienceType::DefaultExperience) => Experience::OpenSystems(os_experiences_all()),
        Some(ExperienceType::MiniExperience) => Experience::OpenSystems(os_experiences_mini()),
        Some(&_) | None => Experience::OpenSystems(os_experiences_default()),
    }
}

fn os_experiences_default() -> Vec<Element> {
    os_experiences_all()
}

fn os_experiences_all() -> Vec<Element> {
    vec![
        os_task_01(),
        os_task_02(),
        separation_between_sections(),
        os_accomplishment_01(),
        os_accomplishment_02(),
    ]
}

fn os_experiences_mini() -> Vec<Element> {
    vec![
        os_task_01(),
        separation_between_sections(),
        os_accomplishment_01(),
    ]
}

fn experience_os_head() -> Element {
    Element::UserDefined(
        r#"
        \cventry{🏢 Open Systems} % Organization
        {💻 Site Reliability Engineer} % Job title
        {Remote 📍} % Location
        {Oct. 2021 – Feb. 2022 📆} % Date(s)
        {\begin{cvitems}
    "#
        .to_string(),
    )
}

fn os_task_01() -> Element {
    Element::UserDefined(
        r#"
        \item {Created a project integrating with a custom ticketing system database, successfully
        \textbf{reducing false positive alerts} and enhancing the precision of incident response
        mechanisms.}
    "#
        .to_string(),
    )
}

fn os_task_02() -> Element {
    Element::UserDefined(
        r#"
        \item {\textbf{Engineered Helm templates} to ensure consistent and reproducible deployments
        of applications within Kubernetes clusters, thereby improving the reliability and
        efficiency of deployment processes.}
    "#
        .to_string(),
    )
}

fn os_accomplishment_01() -> Element {
    Element::UserDefined(
        r#"
        \hll{🎯 Implemented a proactive monitoring system that significantly reduced production
        incidents and \textbf{improved mean time to resolution (MTTR)}, increasing system
        reliability and customer satisfaction.}
    "#
        .to_string(),
    )
}

fn os_accomplishment_02() -> Element {
    Element::UserDefined(
        r#"
        \hll{🎯 Achieved significantly reduced \textbf{false positive alerts} and streamlined
        application deployment in Kubernetes, enhancing system reliability and operational
        efficiency through targeted improvements.}
    "#
        .to_string(),
    )
}
