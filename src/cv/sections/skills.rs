use latex::Element;

pub fn skills(selection: &str) -> Vec<Element> {
    match selection {
        "main" => vec![main_skills()],
        "details" => {
            vec![
                skills_devops(),
                skills_python(),
                skills_networking(),
                skills_scripting(),
                skills_tools(),
                skills_operating_systems(),
                skills_hardware(),
                skills_microcontrollers(),
            ]
        }
        "management" => {
            vec![
                skills_devops(),
                skills_python(),
                skills_networking(),
                skills_management(),
                skills_tools(),
                skills_learnings(),
            ]
        }
        _ => panic!("Invalid selection"),
    }
}

fn main_skills() -> Element {
    Element::UserDefined(
        r#"
\cvsection{📋 Skills}{
    \begin{cvskills}
        \cvskill{Areas of Expertise}{%
            \textbf{Solution Design and Implementation} |
            \textit{Infrastructure Optimisation} |
            \textbf{System Reliability Engineering} |
            \textit{AWS Cloud Services} |
            \textbf{Kubernetes Management} |
            \textit{GitOps Implementation} |
            \textbf{Continuous Integration/Continuous Deployment (CI/CD)} |
            \textit{Application Scalability} |
            \textbf{Monitoring, Logging and Metrics Aggregation} |
            \textit{Python \& Rust Development} |
            \textbf{Terraform, Terragrunt and Infrastructure as Code (IaC)} |
            \textit{Docker and Containerisation} |
            \textbf{Helm Template Engineering} |
            \textit{Prometheus and Grafana} |
            \textbf{Agile Methodologies} |
            \textit{Process Automation} |
            \textbf{Test-Driven Development (TDD)} |
            \textit{Requirements Gathering} |
            \textbf{Documentation Enhancement} |
            \textit{Incident Response Manageme}nt
        }
    \end{cvskills}
}
"#
        .to_string(),
    )
}

fn skills_operating_systems() -> Element {
    Element::UserDefined(
        r#"
\cvskill{Operating Systems} % Category
{Linux (Debian, ArchLinux, CentOS, Ubuntu\ldots), PFSense, OPNSense, Windows, MacOS} % Skills
"#
        .to_string(),
    )
}

fn skills_scripting() -> Element {
    Element::UserDefined(r#"
\cvskill{Scripting} % Category
{Bash, \texttt{awk}, \texttt{sed}, \texttt{grep}, \texttt{dd}, \texttt{ps}, \texttt{regex} (command-line enthusiast)} % Skills
    "#
        .to_string())
}

fn skills_networking() -> Element {
    Element::UserDefined(
        r#"
\cvskill{Networking \& Protocols} % Category
{OpenVPN, Cert-Manager, External-DNS, VLANs, NAT, Route tables, NACLs, \texttt{iptables}} % Skills
    "#
        .to_string(),
    )
}

fn skills_microcontrollers() -> Element {
    Element::UserDefined(
        r#"
\cvskill{Microcontrollers} % Category
{Espressif ESP8266, NXP LPC 1769, NXP LPC11U48, Arduino Uno, FPGA Bus Avalon.}
    "#
        .to_string(),
    )
}

fn skills_hardware() -> Element {
    Element::UserDefined(
        r#"
\cvskill{Hardware \& Embed devices} % Category
{PCB prototyping, PCB soldering, Firmware flashing, Micropython for Embed Devices, FPGA (VHDL)} % Skills

    "#
        .to_string())
}

fn skills_management() -> Element {
    Element::UserDefined(
        r#"
\cvskill{Management} % Category
{SCRUM Agile methodology, COBIT Certification, Business Analysis, KAIZEN Methodology, LEAN Philosophy} % Skills
    "#
        .to_string())
}

fn skills_tools() -> Element {
    Element::UserDefined(
        r#"
// %\cvskill{Tools} % Category
// %{RabbitMQ, PostgresQL, MariaDB, RDF4J (SPARQL Graph), MosquittoMQTT} % Skills
    "#
        .to_string(),
    )
}

fn skills_learnings() -> Element {
    Element::UserDefined(
        r#"
\cvskill{Ongoing Learning} % Category
{Python ML tools, ELK Stack, Mail server config (dovecot, postfix), Rust} % Skills
    "#
        .to_string(),
    )
}

fn skills_devops() -> Element {
    Element::UserDefined(
        r#"
\cvskill{DevOps} % Category
{Docker, Ansible, Gitlab CI/CD, Grafana, Kibana, Logstash}
    "#
        .to_string(),
    )
}

fn skills_python() -> Element {
    Element::UserDefined(
        r#"
\cvskill{Python} % Category
{Flask, Celery, SQLAlchemy, Django, LXML, NumPy, SciPy, SciKit, Jupyter, VirtualEnvs}
    "#
        .to_string(),
    )
}
