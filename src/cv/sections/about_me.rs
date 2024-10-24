use latex::Element;

pub fn about_me() -> Element {
    Element::UserDefined(r#"
\cvsection{💡Who Am I?}{
    \justifying{
        \fontsize{11pt}{2pt}{\bodyfontlight\upshape\color{graytext}{I am a highly
                skilled and versatile professional \textbf{\myposition} with extensive experience in managing and setting up large
                \textbf{Kubernetes production clusters} across all major cloud
                providers. Expertise includes leveraging best practices such as
                \textbf{Test-Driven Development}, \textbf{Infrastructure as Code},
                \textbf{Observability}, and \textbf{Zero Trust security}.
                \\

                Adept at \textbf{implementing} and \textbf{optimizing infrastructure},
                \textbf{automating processes}, and enhancing system reliability using
                advanced technologies like \textbf{Pulumi}, \textbf{AWS SDK/AWS CDK},
                \textbf{ArgoCD} and \textbf{Python}. Proven track record in managing
                large-scale production deployments, improving operational efficiency,
                and ensuring \textbf{high system uptime} and \textbf{stability}. Strong
                background in developing scalable \textbf{cloud infrastructure},
                creating robust \textbf{monitoring systems}, and streamlining incident
                response processes.
                \\

                Exceptional \textbf{problem-solving abilities}, a proactive mindset,
                and a commitment to continuous improvement. Seeking new opportunities
                to leverage expertise in \textbf{cloud infrastructure management}, and
                system reliability to \textbf{deliver impactful solutions}.

            }% color
        }% fontsize
    }% justifying
}% letter section
"#
        .to_string())
}
