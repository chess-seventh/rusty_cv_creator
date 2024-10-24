use latex::Element;

pub fn experience() -> Element {
    Element::UserDefined(
        r#"
\cvsection{👔 Career Summary}{
    \begin{cventries}

        \cventry{🏢 Piva Consulting} % Organization
        {💻 Platform Engineer Consultant - Freelance DevOps \& Site Reliability Engineer} % Job title
        {Remote 📍} % Location
        {Mar. 2024 – Present 📆} % Date(s)
        {\begin{cvitems}
                \item{🎯 Bridging the gap between development and operations teams, fostering seamless collaboration and communication.}
                \item{🎯 Implementing efficient workflows and advocating for best practices, driving enhanced software delivery processes, resulting in improved system reliability and accelerated deployment cycles.}
            \end{cvitems}
        \vspace{-5mm}
        }

        \cventry{🏢 ZF Group} % Organization
        {💻 Senior Platform Engineer} % Job title
        {Remote 📍} % Location
        {Feb. 2022 – Aug. 2024 📆} % Date(s)
        {\begin{cvitems}
                \item {Deploys \textbf{centralised ArgoCD} across all AWS EKS accounts, standardising the \textbf{GitOps} approach and managing over 50 services per cluster, effectively handling approximately 1300 applications.}
                % \item {Implements the \textbf{Mend Renovate} bot for Cloud Platform Engineering teams, ensuring applications, libraries, and modules are \textbf{updated to the latest stable versions}, thereby maintaining software stability and security.}
                \item {Develops a \textbf{Rust application} to enable smooth connectivity for developers to private VPCs, EKS clusters, and endpoints, with supporting infrastructure deployed using \textbf{AWS CDK in Python}.}
                \item {Enhances the developer platform by creating and contributing to the \textbf{central API} for Cloud Platform Engineering’s \textbf{Internal Developer Platform} and creating a command line tool, utilising \textbf{TDD methodology} to achieve a 99\% test code coverage.}
                \item{Centralises \textbf{metrics aggregation} and Grafana dashboards \textbf{across 20 AWS EKS} clusters by rolling out \textbf{Thanos} for all \textbf{Kube-Prometheus-Stacks}, significantly enhancing the platforms’ \textbf{observability} with monitoring and logging coverage.}
                % \item {Revises internal \textbf{documentation} using the \textbf{Divio Documentation framework}, improving the usability and technical clarity of internal tools, thereby \textbf{improving the onboarding process} for new developers.}
                \\
                \newline
                \hll{🎯 Designed and rolled-out a \textbf{centralised metrics aggregator}, enabling real-time incident tracking and performance issue analysis, greatly improving \textbf{platform observability}, stability and operational efficiency.}
                \hll{🎯 Optimised \textbf{GitOps workflows}, automating manual tasks and \textbf{reducing team chore time by 80\%}, enhancing deployment accuracy and team productivity through continuous integration checks.}
                \hll{🎯 Created, and enhanced central API capabilities, \textbf{cutting down} the time needed for AWS ECR repository creation \textbf{by 90\%, eliminating human errors} and improving \textbf{automation efficiency}.}
            \end{cvitems}
        }

        % \vspace{-3mm}
        % \noindent{\color{line-color}\rule{\linewidth}{1pt}}

        \cventry{🏢 Open Systems} % Organization
        {💻 Site Reliability Engineer} % Job title
        {Remote 📍} % Location
        {Oct. 2021 – Feb. 2022 📆} % Date(s)
        {\begin{cvitems}
                \item {Created a project integrating with a custom ticketing system database, successfully \textbf{reducing false positive alerts} and enhancing the precision of incident response mechanisms.}
                \item {\textbf{Engineered Helm templates} to ensure consistent and reproducible deployments of applications within Kubernetes clusters, thereby improving the reliability and efficiency of deployment processes.}
                % \item {Completed extensive internal training on edge security, SD-WAN, firewalls, secured web, and internal tools to be well-prepared for on-call support and assist L3 users during incidents.}
                \\
                \newline
                \hll{🎯 Implemented a proactive monitoring system that significantly reduced production incidents and \textbf{improved mean time to resolution (MTTR)}, increasing system reliability and customer satisfaction.}
                \hll{🎯 Achieved significantly reduced \textbf{false positive alerts} and streamlined application deployment in Kubernetes, enhancing system reliability and operational efficiency through targeted improvements.}
            \end{cvitems}
        }

        % \vspace{-3mm}
        % \noindent{\color{line-color}\rule{\linewidth}{1pt}}

        \cventry{🏢 Bestmile} % Organization
        {💻 Site Reliability Engineer} % Job title
        {Remote 📍} % Location
        {Jul. 2019 – Oct. 2021 📆} % Date(s)
        {\begin{cvitems}
                \item{Applied \textbf{D.R.Y. principles} using \textbf{Terraform} modules and \textbf{Terragrunt}, creating reusable configurations that streamlined infrastructure management and deployment processes.}
                \item{\textbf{Automated infrastructure deployments} with Atlantis and \textbf{GitOps} principles, enhancing consistency and efficiency while reducing manual intervention in deployment workflows.}
                \item{\textbf{Migrated} environments \textbf{from GCP and Azure to AWS}, including Kubernetes clusters to AWS EKS and Apache Kafka to AWS MSK, ensuring improved performance and scalability.}
                \item{\textbf{Managed Kubernetes clusters} by focusing on maintenance, security, and debugging, ensuring \textbf{service stability and high availability} across the infrastructure.}
                \item{Established logging and monitoring systems using \textbf{Prometheus}, \textbf{Grafana}, \textbf{ElasticSearch}, \textbf{Kibana}, \textbf{Logstash}, and \textbf{Filebeat}, enhancing observability and providing actionable insights into system performance.}
                \item{\textbf{Implemented CI/CD} pipelines with Bitbucket and \textbf{Codefresh}, using \textbf{Helm} and \textbf{Helmfile} for efficient package management, keeping Kubernetes base services up to date and enhancing deployment reliability.}
                % \item{\textbf{Administered VPN bastion} hosts on Linux with bash scripts, \textbf{AWS-Packer}, and \textbf{Ansible}, automating configuration management and improving secure access protocols.}
                \\
                \newline
                \hll{🎯 Achieved \textbf{exceptional system uptime} by implementing automated monitoring and alerting tools, significantly \textbf{reducing downtime} and ensuring continuous service availability.}
                \hll{🎯 Successfully established, maintained, and enhanced a \textbf{robust infrastructure from the ground up}, adhering to Site Reliability Engineering (SRE) best practices.}
                \hll{🎯 Successfully \textbf{scaled infrastructure} to handle substantial increases in user traffic during peak times, ensuring a \textbf{smooth user experience without service interruptions}.}
                \hll{🎯 Streamlined incident response by developing and deploying a robust incident management framework, including runbooks and automated remediation processes, leading to quicker resolution of issues.}
            \end{cvitems}}

        % \vspace{-3mm}
        % \noindent{\color{line-color}\rule{\linewidth}{1pt}}

        \cventry{🏢 Pictet Private Banking} % Organization
        {💻 Python Developer} % Job title
        {Geneva Switzerland 📍} % Location
        {May. 2019 – Jul. 2019 📆} % Date(s)
        {\begin{cvitems}
                \item {Developed and implemented an \textbf{Object-Relational Mapping} system for the Neo4J database within the project scope, utilizing the latest \textbf{Python} libraries to enhance data access and manipulation efficiency.}
                \item {Refactored critical components of legacy code, focusing on optimizing performance and responsiveness. Implemented \textbf{Python} best practices to modernize the codebase, resulting in more maintainable and efficient software.}
                % \item {Enhanced the backend system's reactiveness, significantly improving the user experience and operational efficiency. This optimization played a crucial role in supporting product owners in their decision-making processes by providing faster and more reliable data insights.}
                \\
                \newline
                \hll{🎯 Enhancing backend reactiveness significantly supported product owners in their decision-making processes, leading to more informed and timely business decisions.}
            \end{cvitems}
        }

        % \vspace{-3mm}
        % \noindent{\color{line-color}\rule{\linewidth}{1pt}}

        \cventry{🏢 European Broadcasting Union} % Organization
        {💻 Python Developer} % Job title
        {Geneva Switzerland 📍} % Location
        {Jun. 2017 – May. 2019 📆} % Date(s)
        {\begin{cvitems}
                \item {Developed the backend application for the \textbf{European Championships 2018}, enabling live ingestion and streaming of sports data events to partners.}
                \item {Led a production pilot using \textbf{RDF4J} Semantic Database for live sports streams, creating a Python-based \textbf{REST API} with \textbf{flask} and an asynchronous backend with \textbf{RabbitMQ}, \textbf{celery}, and \textbf{lxml} for \textbf{XML} to \textbf{RDF} conversion.}
                % \item {Containerized the project using \textbf{docker} and \textbf{docker-compose}, and deployed it on a \textbf{docker-swarm} cluster, ensuring scalability and efficient operations.}
                \\
                \newline
                \hll{🎯 Production pilot during European Championships 2018 (Glasgow and Berlin), ingest of Live Sports Data.}
            \end{cvitems}
        }

        % \vspace{-5mm}
        % \noindent{\color{line-color}\rule{\linewidth}{1pt}}

        \cventry{🏢 Ducommun Dit Boudry Software Consulting} % Organization
        {💻 Sofware Developer Jr.} % Job title
        {Geneva, Switzerland 📍} % Location
        {Feb. 2017 – May. 2017 📆} % Date(s)
        {\begin{cvitems}
                % \item {Developed and deployed a comprehensive web application, including backend, frontend, and mobile apps for Android, iOS, and Windows}
                \item {Utilized \textbf{Scala} frameworks (\textbf{Play\! Framework} and \textbf{Slick ORM}) for backend development, implemented a simple \textbf{akka} Actor Model, and \textbf{PostgreSQL} as Database. All components were containerized using custom \textbf{docker} images.}
                \item {Created the frontend with \textbf{Angular2 JS} and \textbf{Typescript}, employing the Observer Pattern for efficient state management.}
                \\
                \newline
                \hll{🎯 Created an end-to-end application for an event with admin roles, user modules and asynchronous backend.}
            \end{cvitems}
        }

    \end{cventries}
}
    "#
    .to_string())
}

pub fn experience_hospitality() -> Element {
    Element::UserDefined(
        r#"

        % \noindent
        % {\color{red} \rule{\linewidth}{2mm}}

        %––––––––––––––––––––––––––––––––––––––––––––––––

        % \cventry{Business Process Improvement Manager Jr.} % Job title
        % {Gate Gourmet} % Organization
        % {Geneva \& Zurich, Switzerland} % Location
        % {2013 – 2016} % Date(s)
        % {\begin{cvitems}
        % \item {Creation, development and deployment of internal tools\: Facility Management Tool, Work Allocation Tool, Allergens Replacement Reporting System. All tools coded in \textbf{VBA} Excel. User–friendly interfaces and usage.}
        % \item {\textbf{SAP} Super User for Switzerland (total 3 Units). Implementation of new features for SAP 6.0 Switzerland.}
        % \item {\textbf{SAP (MM/SD/FI)} reports creation for Finance / Supply Chain / Sales and material control.}
        % \\
        % \hll{Create IT tools to connect humans’ inputted data in order to improve its quality.}
        % \end{cvitems}
        % }

        % %––––––––––––––––––––––––––––––––––––––––––––––––
        %
        % \cventry{Back–Office Project Leader} % Job title
        % {Gate Gourmet} % Organization
        % {Geneva, Switzerland} % Location
        % {2014} % Date(s)
        % {\begin{cvitems}
        % \item {Create and implement “Data Collection and Analysis tools” VBA project. Train, document and handover to another supervisor.}
        % \item {Actively participated to the overhauling of Non–Food Dept.\: Work area layout – working method – work allocation – working document – staff training.}
        % \item {Created and implement “IFBL (Internal Feedback Loop)” VBA Project, to trace operational issues and grant follow–up.}
        % \\
        % %\begin{tcolorbox}[colback=gray,leftrule=1pt,rightrule=1pt,toprule=2pt,bottomrule=2pt]
        % \hll{Productivity was improved by 25\% – Customer satisfaction up to 100\%.}
        % %\end{tcolorbox}
        % \end{cvitems}
        % }
        %
        %––––––––––––––––––––––––––––––––––––––––––––––––

        %\cventry{Back–Office Operations Supervisor} % Job title
        %{Gate Gourmet} % Organization
        %{Geneva, Switzerland} % Location
        %{2013} % Date(s)
        %{\begin{cvitems}
        %\item {VBA Tools created in GVA are now used by EasyJet Fraud \& Loss Europe, and other units in Europe.}
        %\item {Creation of control tools to optimize material stock and consumption. Performance analysis.}
        %\item {Implementation of new workflow organization and Internal Delivery System (Kanban \& Automatic Kanban).}
        %\\
        %\hll{Wastage has been reduced by 20\% – achieved stock variance of +/– 1.0\%.}
        %\end{cvitems}
        %}

        %%––––––––––––––––––––––––––––––––––––––––––––––––

        %\cventry{Senior Sales Representative} % Job title
        %{TAG Aviation – Absolute Taste Catering} % Organization
        %{Geneva, Switzerland} % Location
        %{2010 – 2013} % Date(s)
        %{\begin{cvitems}
        %\item {Hired during the startup phase, very hands on from quality control to sales to end customer.}
        %\item {In charge of the day to day operation and catering to VIP Flight crew and customers.}
        %\item {LEAN Management and KAIZEN Implementations.}
        %\\
        %\hll{Absolute Taste took over 40\% of GVA’s market share in less than a year and now leads Geneva airports’ VIP catering}
        %\end{cvitems}
        %}

        %––––––––––––––––––––––––––––––––––––––––––––––––

        % \cventry
        % {Operations Supervisor} % Job title
        % {Sodexo} % Organization
        % {Port–Gentil, Gabon} % Location
        % {Jan. 2010 – Jan. 2010} % Date(s)
        % { % Description(s) of tasks/responsibilities
        % \begin{cvitems}
        % \item {End–to–end follow–up of ‘country’ deliveries to sites. Remote stock control. Complex Excel Documents.}
        % \item {Supply Chain / Site Food + Non–Food Consumption / Billing. Data management of sites.}
        % \item {Coordination, supervision and management of the Learning Center site. (Stock Management, Supply Chain, Excel, Word).}
        % \end{cvitems}
        % }
        %
        % %––––––––––––––––––––––––––––––––––––––––––––––––
        %
        % \cventry
        % {Customer Site Operations Supervisor} % Job title
        % {Sodexo} % Organization
        % {Luanda, Angola} % Location
        % {Jul. 2007 – Jan. 2008} % Date(s)
        % { % Description(s) of tasks/responsibilities
        % \begin{cvitems}
        % \item {Duty Manager for the computers and servers of the Zone Onshore IT. Server Maintenance \& Backups}
        % \item {Responsible for the “Housekeeping Training course”. Daily courses for groups of 5 employees.}
        % \item {Assist the Site Supervisor in Administrative tasks for invoicing and Stock Management}
        % \\
        % \center\colorbox{gray}{Stock Control \& Local deliveries processes reviewed to decreased stock variances \& discrepancies by 50\%}
        % \end{cvitems}
        % }

        %––––––––––––––––––––––––––––––––––––––––––––––––
    "#
    .to_string())
}
