use crate::cv::helpers::{separation_between_sections, Experience, ExperienceType, LatexCvExperienceEntry};
use latex::Element;

pub fn antony(tasks_selector: Option<&ExperienceType>) -> Vec<Element> {
    LatexCvExperienceEntry::new(
        experience_antony_head(),
        experience_antony_body(tasks_selector).compile_elements()
    )
        .compile_elements()
}

fn experience_antony_body(tasks_selector: Option<&ExperienceType>) -> Experience {
    match tasks_selector {
        Some(ExperienceType::DefaultExperience | &_) | None => Experience::Antony(antony_all()),
    }
}

fn antony_all() -> Vec<Element> {
    vec![
        antony_task_01(),
        antony_task_02(),
        antony_task_03(),
        separation_between_sections(),
        antony_accomplishment_01(),
    ]
}

fn antony_task_01() -> Element {
    Element::UserDefined(
        r#"
                \item {Developed and deployed a comprehensive web application, including backend,
                frontend, and mobile apps for Android, iOS, and Windows}
    "#
        .to_string(),
    )
}

fn antony_task_02() -> Element {
    Element::UserDefined(
        r#"
                \item {Utilized \textbf{Scala} frameworks (\textbf{Play\! Framework} and
                \textbf{Slick ORM}) for backend development, implemented a simple \textbf{akka}
                Actor Model, and \textbf{PostgreSQL} as Database. All components were containerized
                using custom \textbf{docker} images.}
    "#
        .to_string(),
    )
}

fn antony_task_03() -> Element {
    Element::UserDefined(
        r#"
                \item {Created the frontend with \textbf{Angular2 JS} and \textbf{Typescript},
                employing the Observer Pattern for efficient state management.}
    "#
        .to_string(),
    )
}

fn antony_accomplishment_01() -> Element {
    Element::UserDefined(
        r#"
                \item {🎯 Created an end-to-end application for an event with admin roles, user
                modules and asynchronous backend.}
    "#
        .to_string(),
    )
}

fn experience_antony_head() -> Element {
    Element::UserDefined(
        r#"
        \cventry{🏢 Ducommun Dit Boudry Software Consulting} % Organization
        {💻 Sofware Developer Jr.} % Job title
        {Geneva, Switzerland 📍} % Location
        {Feb. 2017 – May. 2017 📆} % Date(s)
        {\begin{cvitems}
    "#
        .to_string(),
    )
}

    // Element::UserDefined(
    //     r#"
    //             % \item {Developed and deployed a comprehensive web application, including backend, frontend, and mobile apps for Android, iOS, and Windows}
    //             \item {Utilized \textbf{Scala} frameworks (\textbf{Play\! Framework} and \textbf{Slick ORM}) for backend development, implemented a simple \textbf{akka} Actor Model, and \textbf{PostgreSQL} as Database. All components were containerized using custom \textbf{docker} images.}
    //             \item {Created the frontend with \textbf{Angular2 JS} and \textbf{Typescript}, employing the Observer Pattern for efficient state management.}
    //             \\
    //             \newline
    //             \hll{🎯 Created an end-to-end application for an event with admin roles, user modules and asynchronous backend.}
    // "#
    //     .to_string(),
    // )
