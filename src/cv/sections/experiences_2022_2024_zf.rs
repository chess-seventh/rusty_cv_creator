use crate::cv::helpers::{Experience, ExperienceType, LatexCvExperienceEntry};
use latex::Element;
use crate::cv::helpers::separation_between_sections;

pub fn zf(tasks_selector: Option<&ExperienceType>) -> Vec<Element> {
    LatexCvExperienceEntry::new(
        experience_zf_head(),
        experience_zf_body(tasks_selector).compile_elements()
    ).compile_elements()
}

fn experience_zf_body(tasks_selector: Option<&ExperienceType>) -> Experience {
    match tasks_selector {
        Some(ExperienceType::DefaultExperience) => Experience::Zf(zf_experiences_all()),
        Some(ExperienceType::MiniExperience) => Experience::Zf(zf_experiences_mini()),
        Some(&_) | None => Experience::Zf(zf_experiences_default()),
    }
}


fn zf_experiences_all() -> Vec<Element> {
    vec![
        zf_task_01(),
        zf_task_02(),
        zf_task_03(),
        zf_task_04(),
        zf_task_05(),
        zf_task_06(),
        separation_between_sections(),
        zf_accomplishment_01(),
        zf_accomplishment_02(),
        zf_accomplishment_03(),
    ]
}

fn zf_experiences_mini() -> Vec<Element> {
    vec![
        zf_task_01(),
        zf_task_03(),
        zf_task_05(),
        separation_between_sections(),
        zf_accomplishment_01(),
        zf_accomplishment_02(),
        zf_accomplishment_03(),
    ]
}

fn zf_experiences_default() -> Vec<Element> {
    vec![
        zf_task_01(),
        zf_task_03(),
        zf_task_04(),
        zf_task_05(),
        separation_between_sections(),
        zf_accomplishment_01(),
        zf_accomplishment_02(),
        zf_accomplishment_03(),
    ]
}

fn experience_zf_head() -> Element {
    Element::UserDefined(
        r#"\cventry{🏢 ZF Group} % Organization
        {💻 Senior Platform Engineer} % Job title
        {Remote 📍} % Location
        {Feb. 2022 – Aug. 2024 📆} % Date(s)
        {\begin{cvitems}"#.to_string(),
    )
}

fn zf_task_01() -> Element {
    Element::UserDefined(
        r#"\item {Deploys \textbf{centralised ArgoCD} across all AWS EKS accounts,
                standardising the \textbf{GitOps} approach and managing over 50 services per
                cluster, effectively handling approximately 1300 applications.}"#.to_string(),
    )
}

// Optional
fn zf_task_02() -> Element {
    Element::UserDefined(
        r#"\item {Implements the \textbf{Mend Renovate} bot for Cloud Platform Engineering
                teams, ensuring applications, libraries, and modules are \textbf{updated to the
                latest stable versions}, thereby maintaining software stability and security.}"#.to_string(),
    )
}

fn zf_task_03() -> Element {
    Element::UserDefined(
        r#"\item {Develops a \textbf{Rust application} to enable smooth connectivity for
                developers to private VPCs, EKS clusters, and endpoints, with supporting
                infrastructure deployed using \textbf{AWS CDK in Python}.}"#.to_string(),
    )
}

fn zf_task_04() -> Element {
    Element::UserDefined(
        r#"\item {Enhances the developer platform by creating and contributing to the
                \textbf{central API} for Cloud Platform Engineering’s \textbf{Internal Developer
                Platform} and creating a command line tool, utilising \textbf{TDD methodology} to
                achieve a 99\% test code coverage.}"#.to_string(),
    )
}

fn zf_task_05() -> Element {
    Element::UserDefined(
        r#"\item{Centralises \textbf{metrics aggregation} and Grafana dashboards
                \textbf{across 20 AWS EKS} clusters by rolling out \textbf{Thanos} for all
                \textbf{Kube-Prometheus-Stacks}, significantly enhancing the platforms’
                \textbf{observability} with monitoring and logging coverage.}"#.to_string(),
    )
}

// Optional
fn zf_task_06() -> Element {
    Element::UserDefined(
        r#"\item {Revises internal \textbf{documentation} using the \textbf{Divio
                Documentation framework}, improving the usability and technical clarity of internal
                tools, thereby \textbf{improving the onboarding process} for new developers.}"#.to_string(),
    )
}

fn zf_accomplishment_01() -> Element {
    Element::UserDefined(
        r#"\hll{🎯 Designed and rolled-out a \textbf{centralised metrics aggregator}, enabling
                real-time incident tracking and performance issue analysis, greatly improving
                \textbf{platform observability}, stability and operational efficiency.}"#.to_string(),
    )
}

fn zf_accomplishment_02() -> Element {
    Element::UserDefined(
        r#"\hll{🎯 Optimised \textbf{GitOps workflows}, automating manual tasks and
                \textbf{reducing team chore time by 80\%}, enhancing deployment accuracy and team
                productivity through continuous integration checks.}"#.to_string(),
    )
}

fn zf_accomplishment_03() -> Element {
    Element::UserDefined(
        r#"\hll{🎯 Created, and enhanced central API capabilities, \textbf{cutting down} the
                time needed for AWS ECR repository creation \textbf{by 90\%, eliminating human
                errors} and improving \textbf{automation efficiency}.}"#.to_string(),
    )
}

// Element::UserDefined(
//     r#"
//             \item {Deploys \textbf{centralised ArgoCD} across all AWS EKS accounts, standardising the \textbf{GitOps} approach and managing over 50 services per cluster, effectively handling approximately 1300 applications.}
//             % \item {Implements the \textbf{Mend Renovate} bot for Cloud Platform Engineering teams, ensuring applications, libraries, and modules are \textbf{updated to the latest stable versions}, thereby maintaining software stability and security.}
//             \item {Develops a \textbf{Rust application} to enable smooth connectivity for developers to private VPCs, EKS clusters, and endpoints, with supporting infrastructure deployed using \textbf{AWS CDK in Python}.}
//             \item {Enhances the developer platform by creating and contributing to the \textbf{central API} for Cloud Platform Engineering’s \textbf{Internal Developer Platform} and creating a command line tool, utilising \textbf{TDD methodology} to achieve a 99\% test code coverage.}
//             \item{Centralises \textbf{metrics aggregation} and Grafana dashboards \textbf{across 20 AWS EKS} clusters by rolling out \textbf{Thanos} for all \textbf{Kube-Prometheus-Stacks}, significantly enhancing the platforms’ \textbf{observability} with monitoring and logging coverage.}
//             % \item {Revises internal \textbf{documentation} using the \textbf{Divio Documentation framework}, improving the usability and technical clarity of internal tools, thereby \textbf{improving the onboarding process} for new developers.}
//             \\
//             \newline
//             \hll{🎯 Designed and rolled-out a \textbf{centralised metrics aggregator}, enabling real-time incident tracking and performance issue analysis, greatly improving \textbf{platform observability}, stability and operational efficiency.}
//             \hll{🎯 Optimised \textbf{GitOps workflows}, automating manual tasks and \textbf{reducing team chore time by 80\%}, enhancing deployment accuracy and team productivity through continuous integration checks.}
//             \hll{🎯 Created, and enhanced central API capabilities, \textbf{cutting down} the time needed for AWS ECR repository creation \textbf{by 90\%, eliminating human errors} and improving \textbf{automation efficiency}.}
// "#
//     .to_string(),
// )
