use crate::cv::helpers::{separation_between_sections, ExperienceType, Experience, LatexCvExperienceEntry};
use latex::Element;

pub fn ebu(experience_keyword: Option<&ExperienceType>) -> Vec<Element> {
    LatexCvExperienceEntry::new(
        experience_ebu_head(),
        experience_ebu_body(experience_keyword).compile_elements()
    ).compile_elements()
}

fn experience_ebu_body(experience_keyword: Option<&ExperienceType>) -> Experience {
    match experience_keyword {
        Some(ExperienceType::DefaultExperience) => Experience::Ebu(ebu_experiences_all()),
        Some(ExperienceType::MiniExperience) => Experience::Ebu(ebu_experiences_mini()),
        Some(&_) | None => Experience::Ebu(ebu_experiences_default()),
    }
}

fn ebu_experiences_all() -> Vec<Element> {
    vec![
        ebu_task_01(),
        ebu_task_02(),
        ebu_task_03(),
        separation_between_sections(),
        ebu_accomplishment_01(),
    ]
}

fn ebu_experiences_default() -> Vec<Element> {
    vec![
        ebu_task_01(),
        ebu_task_02(),
        separation_between_sections(),
        ebu_accomplishment_01(),
    ]
}

fn ebu_experiences_mini() -> Vec<Element> {
    vec![
        ebu_task_01(),
        separation_between_sections(),
        ebu_accomplishment_01(),
    ]
}

fn experience_ebu_head() -> Element {
    Element::UserDefined(
        r#"
        \cventry{🏢 European Broadcasting Union} % Organization
        {💻 Python Developer} % Job title
        {Geneva Switzerland 📍} % Location
        {Jun. 2017 – May. 2019 📆} % Date(s)
        {\begin{cvitems} % EBU
    "#
        .to_string(),
    )
}


fn ebu_task_01() -> Element {
    Element::UserDefined(
        r#"
                \item {Developed the backend application for the \textbf{European Championships
                2018}, enabling live ingestion and streaming of sports data events to partners.}
    "#
        .to_string(),
    )
}

fn ebu_task_02() -> Element {
        Element::UserDefined(
        r#"
                \item {Led a production pilot using \textbf{RDF4J} Semantic Database for live
                sports streams, creating a Python-based \textbf{REST API} with \textbf{flask} and
                an asynchronous backend with \textbf{RabbitMQ}, \textbf{celery}, and \textbf{lxml}
                for \textbf{XML} to \textbf{RDF} conversion.}
    "#
        .to_string(),
    )
}

fn ebu_task_03() -> Element {
    Element::UserDefined(
        r#"
                \item {Containerized the project using \textbf{docker} and \textbf{docker-compose},
                and deployed it on a \textbf{docker-swarm} cluster, ensuring scalability and
                efficient operations.}
    "#
        .to_string(),
    )
}

fn ebu_accomplishment_01() -> Element {
    Element::UserDefined(
        r#"
                \hll{🎯 Production pilot during European Championships 2018 (Glasgow and Berlin),
                ingest of Live Sports Data.}
    "#
        .to_string(),
    )
}

// Element::UserDefined(
//     r#"
//             \item {Developed the backend application for the \textbf{European Championships 2018}, enabling live ingestion and streaming of sports data events to partners.}
//             \item {Led a production pilot using \textbf{RDF4J} Semantic Database for live sports streams, creating a Python-based \textbf{REST API} with \textbf{flask} and an asynchronous backend with \textbf{RabbitMQ}, \textbf{celery}, and \textbf{lxml} for \textbf{XML} to \textbf{RDF} conversion.}
//             % \item {Containerized the project using \textbf{docker} and \textbf{docker-compose}, and deployed it on a \textbf{docker-swarm} cluster, ensuring scalability and efficient operations.}
//             \\
//             \newline
//             \hll{🎯 Production pilot during European Championships 2018 (Glasgow and Berlin), ingest of Live Sports Data.}
// "#
//     .to_string(),
// )
