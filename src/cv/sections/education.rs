use latex::Element;

pub fn education() -> Element {
    Element::UserDefined(
        r#"
\cvsection{🎓 Education}{
    \begin{cvhonors}
        \cvhonor{B.S. in Computer Science and Engineering} % Degree
        {HES-SO – hepia (evening courses)} % Institution
        {Geneva, Switzerland} % Location
        {2014 - 2018} % Date(s)

        \cvhonor{Bachelor in Hospitality Management and Marketing} % Degree
        {Glion Institute of Higher Education} % Institution
        {Bulle, Switzerland} % Location
        {2005 - 2008} % Date(s)

        % \cvhonor{COBIT Certification} % Degree
        % {ISACA – IT Training Academy} % Institution
        % {Geneva, Switzerland} % Location
        % {2017} % Date(s)
    \end{cvhonors}
}
    "#
        .to_string(),
    )
}

pub fn education_various() -> Element {
    Element::UserDefined(
        r#"
\cvsection{🎓 Education}{
    \begin{cvhonors}
        %–––––––––––––––––––––––––––––––––––––––––––––––– Business Analyst
        %––––––––––––––––––––––––––––––––––––––––––––––––

        % \cvhonor{SCRUM Master and Business Analyst Course} % Degree
        % {IT Training Academy} % Institution
        % {Geneva, Switzerland} % Location
        % {2017} % Date(s)

        %––––––––––––––––––––––––––––––––––––––––––––––––
        %––––––––––––––––––––––––––––––––––––––––––––––––

        %\cventry{Business Analyst Course} % Degree
        %{IT Training Academy} % Institution
        %{Geneva, Switzerland} % Location
        %{2017} % Date(s)
        %{\begin{cvitems}
        %\end{cvitems}
        %}

        %––––––––––––––––––––––––––––––––––––––––––––––––
        %––––––––––––––––––––––––––––––––––––––––––––––––

        %\cventry
        %{French Scientific Baccalaureate} % Degree
        %{International School} % Institution
        %{Ferney–Voltaire, France} % Location
        %{2002 – 2005} % Date(s)
        %{ % Description(s) bullet points
        %\begin{cvitems}
        %\end{cvitems}
        %}

        %%––––––––––––––––––––––––––––––––––––––––––––––––
        %%––––––––––––––––––––––––––––––––––––––––––––––––

        %\cventry
        %{Cambridge First Certificate (FCE)} % Degree
        %{Cambridge} % Institution
        %{Ferney–Voltaire, France} % Location
        %{2001} % Date(s)
        %{ % Description(s) bullet points
        %\begin{cvitems}
        %\end{cvitems}
        %}

        %%––––––––––––––––––––––––––––––––––––––––––––––––
    \end{cvhonors}
}
    "#
        .to_string(),
    )
}
