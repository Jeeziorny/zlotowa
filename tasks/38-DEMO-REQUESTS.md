# Budget Planing
1. Input dates when creating a budget shouldnt be using default calendar. Instead go with approach that we have in `Add Expense` tab `Date` field.
2. When adding budget, make sure that values for each category are values greater than 0.
3. Whole "Planned Expense" tile can be removed from "Overview" tab.
4. Calendar option should be always unavailable right now. We need to work on this feature later.
5. When I clicked "Delete budget" it was automatically deleted without my approval.It needs to wait for me to approve it.
6. People can add multiple budgets. The thing is, that budgets can't overlap, but I liked the idea that people can browse through previous budgets and future ones using arrows "<" and ">".
CALENDAR:
1. This shouldn't be complicated functionality.
2. I'd like it to be as simple as possible
3. Someone uploads ICS, probably as optional step when creating a budget.
4. Application displays SUGGESTIONS above planning the budget.
5. SUGGESTIONS is a list of points that summarises data from the calendar and correlates it with existing history. Probably we need to talk more about how this should choose sugestions.
6. I don't need anything to be stored in database related to calendar. Planned expenses are overkill too IMO, wdyt?


# Dashboard tiles improvements
1. I'd like dashboard widgets to not to be able to change position by default. There should be a slider somewhere in the dashboard will enable "Edit dashboard" mode where each tile can be moved or removed.
2. Currently, if first row contains two tiles with different heights, the third tile in second row will start displaying where the longer widget in the first row finished. I'd like those tiles to be compact and shift close to each other without gaps.

# Reparability
Put a lot of debug logs, so that I can send the app to my friends and help them quickly if something goes bad

# Tutorials
People won't use it if they won't understand this.