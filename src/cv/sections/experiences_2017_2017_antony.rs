use crate::cv::helpers::LatexCvExperienceEntry;
use latex::Element;

fn experience_antony_body() -> Element {
    Element::UserDefined(
        r#"
                % \item {Developed and deployed a comprehensive web application, including backend, frontend, and mobile apps for Android, iOS, and Windows}
                \item {Utilized \textbf{Scala} frameworks (\textbf{Play\! Framework} and \textbf{Slick ORM}) for backend development, implemented a simple \textbf{akka} Actor Model, and \textbf{PostgreSQL} as Database. All components were containerized using custom \textbf{docker} images.}
                \item {Created the frontend with \textbf{Angular2 JS} and \textbf{Typescript}, employing the Observer Pattern for efficient state management.}
                \\
                \newline
                \hll{🎯 Created an end-to-end application for an event with admin roles, user modules and asynchronous backend.}
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

pub fn antony() -> Vec<Element> {
    LatexCvExperienceEntry::new(experience_antony_head(), vec![experience_antony_body()])
        .compile_elements()
}
