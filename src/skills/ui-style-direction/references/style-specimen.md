# Specify a visual language on one task

Open the [queue specimen](../assets/style-specimen.html) locally. Choose a visual family, select orders, move keyboard focus to another control, and process the selection. Compare the same sequence in each family. The Fluent example uses neutral rows and a command accent. The Material example pairs a tonal surface with its foreground. The Apple example groups commands in a distinct lower bar. All three are web interpretations using original values. The specimen runs offline.

For a product style guide, fill in the design with actual components and supported inputs. A dispatch queue illustrates the level of detail that lets another person implement it.

1. **Hierarchy and density.** The operator finds late orders by status, confirms the order number, then chooses a batch. Give comparable values aligned columns. Let a long status wrap before hiding the order number or the command.
2. **Roles and their values.** Map canvas, work surface, primary text, selected row, keyboard focus, and warning to the existing theme. Include an example value or existing token for each role that changes. In a monochrome style, a check can show selection while a distinctive outline travels with focus. In a themed workspace, keep the status word visible when the selected background changes.
3. **Component states.** Show a selected row with focus elsewhere and a focused row outside the selection. A command scopes itself to the selected count. After processing, keep the operator's next reachable action clear. If processing can fail, refer to the product's actual recovery behavior.
4. **Layout and language.** Use content width to decide whether rows stay in columns or stack. Mirror directional navigation for a supported right to left locale. Isolate order identifiers such as `ORD-0217` so letters and digits retain their intended order. Test the translated label next to a real identifier.
5. **Motion and evidence.** If motion links a changed object to its destination, specify the final state that still communicates the change when motion is reduced. Compare one representative screen in the same state at wide and narrow widths. Mark design proposals and renderer observations separately.

For a narrow styling request, apply only the decisions the change can affect. Historical pixel dimensions, example colors, and platform token names need a named source before being described as official.
