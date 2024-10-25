use crate::cv::helpers::{Experience, ExperienceType, LatexCvExperienceEntry};
use latex::Element;
use crate::cv::helpers::separation_between_sections;


pub fn pictet(experience_keyword: Option<&ExperienceType>) -> Vec<Element> {
    LatexCvExperienceEntry::new(experience_pictet_head(),
                                experience_pictet_body(experience_keyword).compile_elements())
        .compile_elements()
}

fn experience_pictet_body(experience_keyword: Option<&ExperienceType>) -> Experience {
    match experience_keyword {
        Some(ExperienceType::DefaultExperience) => Experience::Pictet(pictet_experiences_all()),
        Some(ExperienceType::MiniExperience) => Experience::Pictet(pictet_experiences_mini()),
        Some(&_) | None => Experience::Pictet(pictet_experiences_default()),
    }
}

fn pictet_experiences_default() -> Vec<Element> {
    pictet_experiences_all()
}

fn pictet_experiences_all() -> Vec<Element> {
    vec![
        pictet_task_01(),
        pictet_task_02(),
        separation_between_sections(),
        pictet_accomplishment_01(),
    ]
}

fn pictet_experiences_mini() -> Vec<Element> {
    vec![
        pictet_task_01(),
        separation_between_sections(),
        pictet_accomplishment_01(),
    ]
}

fn pictet_task_01() -> Element {
    Element::UserDefined(
        r#"
        \item {Developed and implemented an \textbf{Object-Relational Mapping} system for the Neo4J
        database within the project scope, utilizing the latest \textbf{Python} libraries to
        enhance data access and manipulation efficiency.}
    "#
        .to_string(),
    )
}

fn pictet_task_02() -> Element {
    Element::UserDefined(
        r#"
        \item {Refactored critical components of legacy code, focusing on optimizing performance
        and responsiveness. Implemented \textbf{Python} best practices to modernize the codebase,
        resulting in more maintainable and efficient software.}
    "#
        .to_string(),
    )
}

fn pictet_accomplishment_01() -> Element {
    Element::UserDefined(
        r#"
        \hll{🎯 Enhancing backend reactiveness significantly supported product owners in their
        decision-making processes, leading to more informed and timely business decisions.}
    "#
        .to_string(),
    )
}

fn experience_pictet_head() -> Element {
    Element::UserDefined(
        r#"
        \cventry{🏢 Pictet Private Banking} % Organization
        {💻 Python Developer} % Job title
        {Geneva Switzerland 📍} % Location
        {May. 2019 – Jul. 2019 📆} % Date(s)
        {\begin{cvitems}
    "#
        .to_string(),
    )
}

// Element::UserDefined(
//     r#"
//             \item {Developed and implemented an \textbf{Object-Relational Mapping} system for the Neo4J database within the project scope, utilizing the latest \textbf{Python} libraries to enhance data access and manipulation efficiency.}
//             \item {Refactored critical components of legacy code, focusing on optimizing performance and responsiveness. Implemented \textbf{Python} best practices to modernize the codebase, resulting in more maintainable and efficient software.}
//             % \item {Enhanced the backend system's reactiveness, significantly improving the user experience and operational efficiency. This optimization played a crucial role in supporting product owners in their decision-making processes by providing faster and more reliable data insights.}
//             \\
//             \newline
//             \hll{🎯 Enhancing backend reactiveness significantly supported product owners in their decision-making processes, leading to more informed and timely business decisions.}
// "#
//     .to_string(),
// )
