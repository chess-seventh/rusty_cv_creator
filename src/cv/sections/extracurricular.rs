use latex::Element;

pub fn extracurricular() -> Element {
    Element::UserDefined(
        r#"
\cvsection{🏅 Extracurricular}{
    \begin{cventries}
        \vspace{-2mm}
        \cventry{Hobbies}{}{}{}{%
            \begin{cvitems}
                \item{\textbf{Spoken Languages}: \textit{Italian}: Fluent - \textit{English}: Fluent - \textit{French}: Fluent.}
                \item{\textbf{HomeLab} with multi-node and multi-arch K3S, GitLab and Gitlab-CI, Nextcloud, Syncthing, Media Center, Tailscale, Home-Assistant\ldots}
                \item{\textbf{Sports}: Scuba Diving, Swimming, Squash, Basketball, my two Dogs.}
                \item{\textbf{Music}: Vinyl Record collector (1960's 1980's).}
                % \item{\textbf{Electronics projects}: Autowatering System for Plants, IoT devices controllings lights and shutters.}
                % \item{\textbf{Hackathon}: Food OpenData: Winning Team, HP Smart Cities, Arkathon - Hacking Health.}
            \end{cvitems}}
    \end{cventries}
}
    "#
        .to_string(),
    )
}
