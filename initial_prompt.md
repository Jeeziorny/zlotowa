## 4ccountant 
Desktop Application for classifying expenses and ploting graphs. In future also budget planner. Gives users ability to interact with its logic through CLI (terminal).

### Components
- **Classification database** - mapping A -> B, where A is regex describing expense title and B is category assigned for all expenses that regex A matched. It can be lightweight database or file. It will be living where the desktop app will be run, so no external connection. Technical decisions are up to you.
- **Available Parsers** - People can paste their expenses from different sources. Each bank will have its own .csv grammar or page layout. Parsers are stored within the source files, can be added through PR to the repo. For MVP parsers need to handle text input, but in later phases image parsers will be needed for parsing receipts.
- **Expense database** - database with expenses. It can be a table, it can be a file. Technical stuff up to you.

### Functionalities
#### Bulk-classify, but no LLM configuration is found

User is asked if they want to proceed with configuring LLM.

**1. Proceed with configuration**:

Steps: 
1. User is prompted to set some kind of key that will integrate the application with users LLM licence. This step needs to do some validation if given key works. Maybe even by sending some dummy request to AI?
2. User is prompted to input example expenses.
    - they can paste snippet with .csv structure,
    - they can paste HTML code of bank page that currently displays expense history (if they are not able to export CSV file).
    - at least three expenses. If user pastes too short snippet, application asks if they are sure they want to proceed with that.
3. Application tries to match the file format with available parsers.
4. If file can't be matched say that application does not support such format yet, go back to step 2.
5. Application uses parser on user input trying to get acceptance from the user about parsed data:
    - app asks if given string is a title,
    - app asks if given string is a date of an expense,
    - app asks if given string is amount of expense. **Maybe user can choose which fields they want to be on parsing output?**
In case it won't be approved by the user, application asks if they are sure and whole process go back to point number 2.

**1. Skip the configuration**

(Go to next phase: Expense upload)

**2. Expense upload ***

Steps:
1. User is presented with popup that asks for data. Data can be:
   - textfield that waits for pasted text,
   - file field that enables drag-and-drop functionality and/or opens the file browser to select the file (file must be .txt or .csv).
2. Input is parsed to format that can be easly handled by application.
3. Automatic clasification.
   1. Parsed expense title is used for automatic expense classification.
   2. Application tries to clasify expenses using classification from the past. Classifications from the past lives in **classification database**.
   3. If there are still expenses to be classified, application prepares a request to LLM asking for clasification. LLM is used only when given expense can't be matched with information from database.
4. User is presented with table view consisting of expenses and their category proposition. 
   1. Top expenses are expenses using data from database,
   2. Below are expenses with clasification retrieved from LLM. 
   3. Below LLM clasifications are duplicates. For MVP duplicates will be recognised as every expense having the same title, amount and date. User can't do anything with duplicates, they are only presented as such and they won't be appended to the **expenses database.**
   4. Each group needs to be presented to user, so that they will know that top expenses are most reliable and mid expenses are LLM expenses.
5. User approves clasification
6. User sees dashboard again.

Additional information:
- User is presented with the progress bar or step in the whole process.
- In later phases there is a possibility of having public classification database. It will serve the same purpose as users classification database, but it will be available in the web (S3 or sth) so that each app could use it as source of truth. In such case classified expenses will be annotated that they are classified used public database.



#### Classify single expense
1. User sees a popup with three fields to be filled:
   1. Date (default today, if they click on date there is some calendar to choose the date),
   2. Expense title (when user types title, application prompts possible clasification using data from clasification database),
   3. Expense amount.
2. User submits the expense
3. Expense is saved to expense database
4. Classification database is updated with new mapping if it's new. Nothing is done if such mapping already exists.

#### Dashboard
1. Graphs. Do the research which graphs are most suitable for such application.
2. User can add a widget to the dashboard. 
3. Dashboard widget is a component, that can be implemented by other people and added to the application dynamically. Widget can be
   1. A graph,
   2. expense statistic (biggest expense this month, most frequent expense and so on..)
   3. In future it can also be some kind of widget related with budget planning.

#### Export to .csv
- User is able to export expense database with their clasification to .csv file.
- this functionality also needs to be open for extension. Users should be able to choose the layout of output file (which columns it will contain).

#### CLI
There are CLI commands that should have same functionalities as the app:
1. llm-conf - process of setting up LLM Api Key through command line.
2. bulk-insert <PATH_TO_CSV_FILE> - this command won't propt for user to configure LLM API KEY in case they don't have it. It will display warning, that api key is not configured and all expenses would need to be classified using classification database and/or manual clasification. bulk-insert command will go through same phases as in GUI: it tries to parse, assign categories (if LLM is configured). Then someone would need to approve this labeling and/or edit it. It's up to you (choose better UX) if this will be done by some kind of temporary file or inside shell.
3. insert - insert only one expense. Asks user for clasification label
4. export <GRAMMAR_FILE> - export classified expense data. GRAMMAR_FILE should define which columns would need to be exported. If you find it better to have grammar defined interactively than through grammar file it's ok, choose better UX approach.
5. dashboard - opens the GUI app dashboard with widgets and data.

Questions:
1. Dashboard: Simple HTML file that can be opened by browser, or more complicated app? Or maybe core is an app and dashboard is simple HTML page? With that approach people using CLI can browse only report.
2. Do you think that there are some places that this app could be more open to contribution like parsers, widgets?
3. Plan the work so that I can use git branches

