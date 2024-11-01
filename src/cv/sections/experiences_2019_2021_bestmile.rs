use crate::cv::helpers::{Experience, ExperienceType, LatexCvExperienceEntry};

use latex::Element;
use crate::cv::helpers::separation_between_sections;

pub fn bestmile(tasks_selector: Option<&ExperienceType>) -> Vec<Element> {
    LatexCvExperienceEntry::new(
        experience_bestmile_head(),
        experience_bestmile_body(tasks_selector).compile_elements(),
    ).compile_elements()
}

fn experience_bestmile_body(tasks_selector: Option<&ExperienceType>) -> Experience {
    match tasks_selector {
        Some(ExperienceType::DefaultExperience) => Experience::Bestmile(bestmile_all()),
        Some(ExperienceType::MiniExperience) => Experience::Bestmile(bestmile_mini()),
        Some(&_) | None => Experience::Bestmile(bestmile_default()),
    }
}

fn bestmile_default() -> Vec<Element> {
    vec![
        bestmile_task_01(),
        bestmile_task_03(),
        bestmile_task_04(),
        bestmile_task_05(),
        bestmile_task_06(),
        separation_between_sections(),
        bestmile_accomplishment_01(),
        bestmile_accomplishment_02(),
        bestmile_accomplishment_03(),
        bestmile_accomplishment_04(),
    ]
}

fn bestmile_all() -> Vec<Element> {
    vec![
        bestmile_task_01(),
        bestmile_task_02(),
        bestmile_task_03(),
        bestmile_task_04(),
        bestmile_task_05(),
        bestmile_task_06(),
        bestmile_task_07(),
        separation_between_sections(),
        bestmile_accomplishment_01(),
        bestmile_accomplishment_02(),
        bestmile_accomplishment_03(),
        bestmile_accomplishment_04(),
    ]
}

fn bestmile_mini() -> Vec<Element> {
    vec![
        bestmile_task_01(),
        bestmile_task_02(),
        bestmile_task_04(),
        bestmile_task_05(),
        bestmile_task_06(),
        separation_between_sections(),
        bestmile_accomplishment_01(),
        bestmile_accomplishment_02(),
        bestmile_accomplishment_03(),
        bestmile_accomplishment_04(),
    ]
}


fn bestmile_task_01() -> Element {
    Element::UserDefined(
        r#"                \item{Applied \textbf{D.R.Y. principles} using \textbf{Terraform} modules and
                \textbf{Terragrunt}, creating reusable configurations that streamlined
                infrastructure management and deployment processes.}"#.to_string(),
    )
}

fn bestmile_task_02() -> Element {
    Element::UserDefined(
        r#"                \item{\textbf{Automated infrastructure deployments} with Atlantis and \textbf{GitOps}
                principles, enhancing consistency and efficiency while reducing manual intervention in
                deployment workflows.}"#.to_string(),
    )
}

fn bestmile_task_03() -> Element {
    Element::UserDefined(
        r#"                \item{\textbf{Migrated} environments \textbf{from GCP and Azure to AWS}, including
                Kubernetes clusters to AWS EKS and Apache Kafka to AWS MSK, ensuring improved
                performance and scalability.}"#.to_string(),
    )
}

fn bestmile_task_04() -> Element {
    Element::UserDefined(
        r#"                \item{\textbf{Managed Kubernetes clusters} by focusing on maintenance, security, and
                debugging, ensuring \textbf{service stability and high availability} across the
                infrastructure.}"#.to_string(),
    )
}

fn bestmile_task_05() -> Element {
    Element::UserDefined(
        r#"                \item{Established logging and monitoring systems using \textbf{Prometheus},
                \textbf{Grafana}, \textbf{ElasticSearch}, \textbf{Kibana}, \textbf{Logstash}, and
                \textbf{Filebeat}, enhancing observability and providing actionable insights into
                system performance.}"#.to_string(),
    )
}

fn bestmile_task_06() -> Element {
    Element::UserDefined(
        r#"                \item{\textbf{Implemented CI/CD} pipelines with Bitbucket and \textbf{Codefresh}, using
                \textbf{Helm} and \textbf{Helmfile} for efficient package management, keeping Kubernetes
                base services up to date and enhancing deployment reliability.}"#.to_string(),
    )
}

fn bestmile_task_07() -> Element {
    Element::UserDefined(
        r#"                \item{\textbf{Administered VPN bastion} hosts on Linux with bash scripts,
                \textbf{AWS-Packer}, and \textbf{Ansible}, automating configuration management and
                improving secure access protocols.}"#.to_string(),
    )
}

fn bestmile_accomplishment_01() -> Element {
    Element::UserDefined(
        r#"                \hll{🎯 Achieved \textbf{exceptional system uptime} by implementing automated
                monitoring and alerting tools, significantly \textbf{reducing downtime} and
                ensuring continuous service availability.}"#.to_string(),
    )
}

fn bestmile_accomplishment_02() -> Element {
    Element::UserDefined(
        r#"                \hll{🎯 Successfully established, maintained, and enhanced a \textbf{robust
                infrastructure from the ground up}, adhering to Site Reliability Engineering (SRE)
                best practices.}"#.to_string(),
    )
}

fn bestmile_accomplishment_03() -> Element {
    Element::UserDefined(
        r#"                \hll{🎯 Successfully \textbf{scaled infrastructure} to handle substantial increases
                in user traffic during peak times, ensuring a \textbf{smooth user experience without
                service interruptions}.}"#.to_string(),
    )
}

fn bestmile_accomplishment_04() -> Element {
    Element::UserDefined(
        r#"                \hll{🎯 Streamlined incident response by developing and deploying a robust incident
                management framework, including runbooks and automated remediation processes, leading
                to quicker resolution of issues.}"#.to_string(),
    )
}


fn experience_bestmile_head() -> Element {
    Element::UserDefined(
        r#"
        \cventry{🏢 Bestmile} % Organization
        {💻 Site Reliability Engineer} % Job title
        {Remote 📍} % Location
        {Jul. 2019 – Oct. 2021 📆} % Date(s)
        {\begin{cvitems}"#.to_string(),
    )
}

// Element::UserDefined(
//     r#"
//             \item{Applied \textbf{D.R.Y. principles} using \textbf{Terraform} modules and \textbf{Terragrunt}, creating reusable configurations that streamlined infrastructure management and deployment processes.}
//             \item{\textbf{Automated infrastructure deployments} with Atlantis and \textbf{GitOps} principles, enhancing consistency and efficiency while reducing manual intervention in deployment workflows.}
//             \item{\textbf{Migrated} environments \textbf{from GCP and Azure to AWS}, including Kubernetes clusters to AWS EKS and Apache Kafka to AWS MSK, ensuring improved performance and scalability.}
//             \item{\textbf{Managed Kubernetes clusters} by focusing on maintenance, security, and debugging, ensuring \textbf{service stability and high availability} across the infrastructure.}
//             \item{Established logging and monitoring systems using \textbf{Prometheus}, \textbf{Grafana}, \textbf{ElasticSearch}, \textbf{Kibana}, \textbf{Logstash}, and \textbf{Filebeat}, enhancing observability and providing actionable insights into system performance.}
//             \item{\textbf{Implemented CI/CD} pipelines with Bitbucket and \textbf{Codefresh}, using \textbf{Helm} and \textbf{Helmfile} for efficient package management, keeping Kubernetes base services up to date and enhancing deployment reliability.}
//             % \item{\textbf{Administered VPN bastion} hosts on Linux with bash scripts, \textbf{AWS-Packer}, and \textbf{Ansible}, automating configuration management and improving secure access protocols.}
//             \\
//             \newline
//             \hll{🎯 Achieved \textbf{exceptional system uptime} by implementing automated monitoring and alerting tools, significantly \textbf{reducing downtime} and ensuring continuous service availability.}
//             \hll{🎯 Successfully established, maintained, and enhanced a \textbf{robust infrastructure from the ground up}, adhering to Site Reliability Engineering (SRE) best practices.}
//             \hll{🎯 Successfully \textbf{scaled infrastructure} to handle substantial increases in user traffic during peak times, ensuring a \textbf{smooth user experience without service interruptions}.}
//             \hll{🎯 Streamlined incident response by developing and deploying a robust incident management framework, including runbooks and automated remediation processes, leading to quicker resolution of issues.}
// "#
//     .to_string(),
// )
