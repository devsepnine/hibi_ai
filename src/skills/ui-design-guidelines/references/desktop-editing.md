# Desktop editing and command scope

**Selection and focus.** Let the user see which objects a command affects while focus moves to a toolbar or inspector. A count in the action label can disambiguate the scope. State whether the command targets selected rows, the visible page, or all filtered results if those sets differ. With a tree, specify whether selecting a folder includes descendants for this particular operation.

**Inspect or stage.** An inspector that follows selection suits repeated edits across objects. For immediate edits, update the identified objects as the user commits a field. For staged edits, keep the draft attached to its original selection and make the pending change visible before the selection changes. Define the behavior of Apply, Cancel and closing the panel for the model the product uses.

**Mixed values and eligibility.** If selected photos have different ratings, show a mixed value. Merely opening the inspector or editing another field must preserve those ratings. After the user chooses a new rating, apply only the edited property to the eligible objects. If the product allows partial success, show which objects changed and which were skipped. If it requires the whole selection to be eligible, explain what blocks the command.

**Command placement.** Keep a frequent command within easy reach while retaining a named route to it. Selection-specific tools can appear when relevant. A dialog belongs to the object or document whose work it blocks. Verify the scope by editing in one document while another remains open.

Source examples include *Object-Oriented Interface Design*, 1993, the inspector panels in *OpenStep User Interface Guidelines*, 1996, the property sheets in *The Windows Interface Guidelines for Software Design*, 1995, and the contextual tabs in *2007 Microsoft Office System User Interface Design Guidelines*. The photo scenario applies these ideas to a new task.
