   1. In bulk upload, when request to llm is done there is no feedback for user that LLM is working now. Some popup with progress bar will be great here.
   2. Match keyword and category textfields in bulk upload are still small. I think we need to put those textfields below title, date and amount so that we can add longer strings. Why you can't add those chips below? Like there will be text field, someone gives input. After they hit finish, textfield is empty but new label is appended below row data. Up to 1 label for now. It's MVP.
   3. In Expenses View "Source" should be replaced with "Annotated by"
   4. "No LLM API key configured. Expenses not matched by rules will need manual categorization." popup needs to be able to be closed by user.
   5. Bulk upload -> Map columns. In order to map columns you need to print only one row of expense. In header of the table I'd like to have some placeholders, so that user will see that they need to take some action to set column types.
   6. I think that in "Expenses" tab there is no need to keep the information about who labelled given expense. Simply drop that from UI.
   7. Before Deleting expense from Expenses tab ask if user is sure.
   8. Why the hell every time I change the app app goes to dashboard???!!!
   9.  When you remove "Source" from expenses table, make "Category" wider so that longer categories fit.
   10. On "Title Cleanup" tab explain what is it about and how it works. Does the title cleanup rules apply also to freshly uploaded expenses in "Bulk upload" view?
   11. Change "Bulk Upload" tab name to "Expense bulk upload".
   12. On Dashboard:
      1. Clicking on Total Expenses opens "Expenses" tab in the app.
      2. Clicking on Transactions opens "Expenses" tab in the app.
      3. Clicking on Categories opens "Categories" tab in the app
   13. In budget planning I'd like to have three tabs:
      1. Overview (default):
         1. Instead of Having "month year" in top right corner it displays current budget date range FROM - TO. For now there will be only one active budget.
      2. "Create +" (Current "Budget").
         1. Creating a budget will be multi step process.
         2. First step will look the same as it is now, but you'll display only categories that were used in previous month.
         3. "Budget" column will have default value in every row equal to Avg.
         4. User need to set the "budget period". It will be a range with start and end date. Budgets can't overlap.
         5. User can add more rows (through button). Clicking "Add more categories" displays popup that displays all categories user has except those that are already in the table.
         6. After everything is setup, user needs to click "Create".
         7. Now "Create +" has greyed UI and displays a message, that current time period already has a budget.
      3. "Calendar".
         1. Uploading ICS works great. The problem is that after uploading ics there is nothing user can do about those meetings.. Maybe there is some way to assign some amount of money they will spent that day? Or is there a better approach?
   