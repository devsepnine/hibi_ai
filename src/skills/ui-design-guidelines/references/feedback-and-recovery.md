# Feedback and recovery

## Match the interruption to the consequence

If a change is easy to reverse, show the new state in place. Offer Undo when the product can restore the previous state. If a command would cause significant, irreversible loss, identify the affected item and consequence before execution. Name the action on the confirmation button and provide a way to leave safely.

For example, archiving a message can offer Undo if the mailbox can restore it. Permanently erasing a shared folder calls for naming the folder and explaining who loses access. Check what happens on cancellation and what the user sees if the operation fails.

Source examples include *Material Design*, 2018, “Confirmation & acknowledgement”, and *GNOME Human Interface Guidelines*, “Alerts”.

## Show what is known about the result

- **Pending.** Show measured progress when the operation reports meaningful units, such as files transferred out of a known total. With an unknown total, identify the ongoing task and use an indeterminate indicator if a wait is noticeable.
- **No content.** An empty library may offer a supported way to add its first item. An empty search should keep the query editable. If loading failed, retain the useful context and show how to recover when recovery is available.
- **Invalid input.** Place the reason and a specific correction beside the field. Keep valid entries while the user corrects the problem.
- **Lost response.** If the server might already have applied a change, preserve the draft where possible and explain that the outcome is unknown. Use an existing status or history view to check the result when available. Offer retry only when repeating the command is safe.

Source examples include *GNOME Human Interface Guidelines*, “Progress Bars”, *Material Design*, 2018, “Empty states”, and *User Experience Interaction Guidelines for Windows 7 and Windows Vista*, 2010, “Error Messages”.
