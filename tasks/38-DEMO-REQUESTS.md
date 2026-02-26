# Budget Planing
1. Input dates when creating a budget shouldnt be using default calendar. Instead go with approach that we have in `Add Expense` tab `Date` field.
2. When adding budget, make sure that values for each category are values greater than 0.
3. Whole "Planned Expense" tile can be removed from "Overview" tab.
4. Calendar option should be always unavailable right now. We need to work on this feature later.
5. When I clicked "Delete budget" it was automatically deleted without my approval.It needs to wait for me to approve it.
6. People can add multiple budgets. The thing is, that budgets can't overlap, but I liked the idea that people can browse through previous budgets and future ones using arrows "<" and ">".

# Navigation Menu design
1. I'm considering following improvement, tell me what do you think from UI/UX perspective.
2. New Menu tabs:
   1. Dashboard (without explicit tab, just people woud need to click on 4ccountant logo)
   2. Single "Expense" tab, where people will have some kind of floating buttons for add single and add multiple that will trigger functionalities of "Add Expense" and "Upload Bulk Expense"
   3. "Categories" left as they are right now
   4. "Budget" left as it is right now.
3. I have some problems with placing "Title cleanup" together with its funcionality. I want this feature probably to be visible as some kind of action that user can do in "Expense" tab.
   1. First approach is to have such functionality for "Automatic title renaming" in "Expense" tab. People could there have exactly the same functionality as they have now (only button placement will change)
   2. Second approach is to have it as it is.
   3. Whatever approach we'd choose I think the good idea is to have expense.display_name and expense.name where display_name is by default == name, but if someone would apply title cleanup it would apply only to display_name so we won't break original title
4. I also don't know how to handle all this automatic title renaming while adding expense / multiple expenses. Should program detect those "ugly" titles and show suggestions for renaming? Probably yes. If so, should it perform mapping to category BEFORE renaming or AFTER? How those suggestions should be presented to the user?
5. This task should focus only on fronend. Some buttons should be replaced, but logic must remain the same.

# Dashboard tiles improvements
1. I'd like dashboard widgets to not to be able to change position by default. There should be a slider somewhere in the dashboard will enable "Edit dashboard" mode where each tile can be moved or removed.
2. Currently, if first row contains two tiles with different heights, the third tile in second row will start displaying where the longer widget in the first row finished. I'd like those tiles to be compact and shift close to each other without gaps.

# Calendar module
1. This shouldn't be complicated functionality.
2. I'd like it to be as simple as possible
3. Someone uploads ICS, probably as optional step when creating a budget.
4. Application displays SUGGESTIONS above planning the budget.
5. SUGGESTIONS is a list of points that summarises data from the calendar and correlates it with existing history. Probably we need to talk more about how this should choose sugestions.

# Reparability
Put a lot of debug logs, so that I can send the app to my friends and help them quickly if something goes bad

# Tutorials
People won't use it if they won't understand this.