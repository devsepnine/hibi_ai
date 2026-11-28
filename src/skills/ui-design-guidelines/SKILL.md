---
name: ui-design-guidelines
description: "Design screen flows, hierarchy, input and recovery across devices. 화면 설계, 반응형 UI, 탐색, 인터랙션 설계."
---

# UI design guidelines

Start with what the person came to do, the content they need to see together, and how they operate the screen. Use the product's existing components and visual language. Apply the decisions directly to the requested wireframe, implementation, or guide.

When a flow change must preserve the product's style, show where the existing controls reappear. For example, a search panel can reuse the product's search field and result rows while changing where selection returns. Use component names from the project when available.

## Establish a visual hierarchy

- Use a realistic record to decide what readers scan first. In an inbox that might be the sender and subject. In a delivery queue it might be the exception and the deadline. Give those fields the strongest text role and align repeated values for comparison.
- Map title, body, and supporting text to the project's type styles. Keep a label close to its value and leave more space between unrelated groups. Try a long translated label and enlarged text before choosing truncation or a fixed row height.
- Assign emphasis by meaning. A filled action can show what to do next, a persistent selection marker can show what is open, and a status label can explain what went wrong. Check that those cues remain distinguishable when they appear together.

These choices draw on *Material Design*, 2018, especially “The type system”, “Spacing methods”, and “Applying color to UI”. Use the target product's tokens and platform units for concrete values. Historical dimensions and gestures belong to their original device and version.

## Choose the interaction

- When layout or input changes across devices, use [navigation and input](references/navigation-and-input.md) to decide what stays visible, how to reach an action, and where the user returns.
- When a command changes data or depends on a service, use [feedback and recovery](references/feedback-and-recovery.md) to design the pending state, the result, and the next action the product can actually perform.
- For persistent inspectors, bulk edits and workspace commands, use [desktop editing](references/desktop-editing.md).
- For pen capture, watch handoff, shared tabletops or headset input, use [device interactions](references/device-interactions.md). Choose the section for the target hardware.

When the request names a visual era, asks for a faithful reconstruction, or needs a product style guide, use `ui-style-direction` if available. Keep the task flow here as the test case for that visual direction.

Walk through the primary task with the target input and one case that could change the design. For a prototype, demonstrate the transition. For a written proposal, describe what the person would see and be able to do.
