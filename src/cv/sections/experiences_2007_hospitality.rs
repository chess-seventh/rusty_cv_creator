use latex::Element;

pub fn experience_hospitality() -> Vec<Element> {
    vec![Element::UserDefined(
        r#"

        % \noindent
        % {\color{red} \rule{\linewidth}{2mm}}

        %––––––––––––––––––––––––––––––––––––––––––––––––

        \cventry{Business Process Improvement Manager Jr.} % Job title
        {Gate Gourmet} % Organization
        {Geneva \& Zurich, Switzerland} % Location
        {2013 – 2016} % Date(s)
        {\begin{cvitems}
        \item {Creation, development and deployment of internal tools\: Facility Management Tool, Work Allocation Tool, Allergens Replacement Reporting System. All tools coded in \textbf{VBA} Excel. User–friendly interfaces and usage.}
        \item {\textbf{SAP} Super User for Switzerland (total 3 Units). Implementation of new features for SAP 6.0 Switzerland.}
        \item {\textbf{SAP (MM/SD/FI)} reports creation for Finance / Supply Chain / Sales and material control.}
        \\
        \hll{Create IT tools to connect humans’ inputted data in order to improve its quality.}
        \end{cvitems}
        }

        % %––––––––––––––––––––––––––––––––––––––––––––––––
        %
        \cventry{Back–Office Project Leader} % Job title
        {Gate Gourmet} % Organization
        {Geneva, Switzerland} % Location
        {2014} % Date(s)
        {\begin{cvitems}
        \item {Create and implement “Data Collection and Analysis tools” VBA project. Train, document and handover to another supervisor.}
        \item {Actively participated to the overhauling of Non–Food Dept.\: Work area layout – working method – work allocation – working document – staff training.}
        \item {Created and implement “IFBL (Internal Feedback Loop)” VBA Project, to trace operational issues and grant follow–up.}
        \\
        \hll{Productivity was improved by 25\% – Customer satisfaction up to 100\%.}
        \end{cvitems}
        }
        %
        %––––––––––––––––––––––––––––––––––––––––––––––––

        \cventry{Back–Office Operations Supervisor} % Job title
        {Gate Gourmet} % Organization
        {Geneva, Switzerland} % Location
        {2013} % Date(s)
        {\begin{cvitems}
        \item {VBA Tools created in GVA are now used by EasyJet Fraud \& Loss Europe, and other units in Europe.}
        \item {Creation of control tools to optimize material stock and consumption. Performance analysis.}
        \item {Implementation of new workflow organization and Internal Delivery System (Kanban \& Automatic Kanban).}
        \\
        \hll{Wastage has been reduced by 20\% – achieved stock variance of +/– 1.0\%.}
        \end{cvitems}
        }

        %%––––––––––––––––––––––––––––––––––––––––––––––––

        \cventry{Senior Sales Representative} % Job title
        {TAG Aviation – Absolute Taste Catering} % Organization
        {Geneva, Switzerland} % Location
        {2010 – 2013} % Date(s)
        {\begin{cvitems}
        \item {Hired during the startup phase, very hands on from quality control to sales to end customer.}
        \item {In charge of the day to day operation and catering to VIP Flight crew and customers.}
        \item {LEAN Management and KAIZEN Implementations.}
        \\
        \hll{Absolute Taste took over 40\% of GVA’s market share in less than a year and now leads Geneva airports’ VIP catering}
        \end{cvitems}
        }

        %––––––––––––––––––––––––––––––––––––––––––––––––

        \cventry
        {Operations Supervisor} % Job title
        {Sodexo} % Organization
        {Port–Gentil, Gabon} % Location
        {Jan. 2010 – Jan. 2010} % Date(s)
        { % Description(s) of tasks/responsibilities
        \begin{cvitems}
        \item {End–to–end follow–up of ‘country’ deliveries to sites. Remote stock control. Complex Excel Documents.}
        \item {Supply Chain / Site Food + Non–Food Consumption / Billing. Data management of sites.}
        \item {Coordination, supervision and management of the Learning Center site. (Stock Management, Supply Chain, Excel, Word).}
        \end{cvitems}
        }
        %
        % %––––––––––––––––––––––––––––––––––––––––––––––––
        %
        \cventry
        {Customer Site Operations Supervisor} % Job title
        {Sodexo} % Organization
        {Luanda, Angola} % Location
        {Jul. 2007 – Jan. 2008} % Date(s)
        { % Description(s) of tasks/responsibilities
        \begin{cvitems}
        \item {Duty Manager for the computers and servers of the Zone Onshore IT. Server Maintenance \& Backups}
        \item {Responsible for the “Housekeeping Training course”. Daily courses for groups of 5 employees.}
        \item {Assist the Site Supervisor in Administrative tasks for invoicing and Stock Management}
        \\
        \center\colorbox{gray}{Stock Control \& Local deliveries processes reviewed to decreased stock variances \& discrepancies by 50\%}
        \end{cvitems}
        }

        %––––––––––––––––––––––––––––––––––––––––––––––––
    "#
    .to_string())]
}
