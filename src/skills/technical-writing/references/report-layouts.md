# Choose a layout by the reader's question

For a longer deliverable, use the reader's main question to choose the opening. Add a heading where someone will return for a particular answer. If a research report also asks for a decision, lead with the decision and give the findings that support it.

| Deliverable | Reader's question | What the reader sees |
|---|---|---|
| Progress or completion report | Where does the work stand? | Lead with the outcome. Give the evidence of completion or the blocker and what will unblock it. Use a checklist when several tasks need tracking. |
| Product proposal or brief | Is this problem worth solving now? | Open with the proposal or the decision requested. Describe the problem and the proposed trial. Present a way to measure its effect as a suggestion until someone agrees to it. |
| Options and recommendation | Which option fits our constraints? | State the recommendation and the deciding criterion. Compare options against the same criteria. Include what happens if each option is chosen and the evidence for differences that matter. Say what would change the recommendation. State any acceptance condition shared by the follow-up actions once above the list. |
| Research report or experiment | What did we learn, and how far does it apply? | State what the sample shows and cite the observation. Report how participants felt when their own accounts support it. Identify what the study leaves open. |
| Architecture or runtime explanation | How do the parts work together? | Follow one input from its first component to its destination. Explain a component's purpose where it first affects the flow. Put each branch condition next to its outcome. Cite the implementation when explaining code. |
| Decision record | Why did we choose this design? | State the decision and its status. Follow with constraints, alternatives considered, and consequences. |
| Implementation plan or ticket | What work will satisfy the request? | State the desired behavior and acceptance criteria. Order the work by dependency. List open decisions where they affect a step. |
| Procedure or tutorial | How do I complete this task? | Put prerequisites before the steps. Number the actions and show what the reader can observe to confirm success. |
| Reference or feature map | Where do I find a setting or behavior? | Organize entries by the name or symptom the reader knows. Use a lookup table for comparable fields. |
| Incident or debugging report | Who is affected and what do we know? | Lead with impact and current state. Use a timeline when sequence matters. Label suspected causes and give the observations behind them. |
| Code review or risk assessment | What needs fixing first? | Order findings by impact. Give the location, the failing condition, and its consequence together. |
| PR, commit, or release note | What changed for me? | Use the required artifact format. Lead with behavior or user impact. Include verification or upgrade steps where relevant. |
| QA handoff or verification report | What should I check? | Pair each scenario with its expected result. Record observed results for checks already run and call out untested paths. |
| Meeting notes or retrospective | What did we decide and who follows up? | Put decisions or lessons first. List agreed actions with owners and dates when known. |

## Instructional documents

Choose a Diátaxis form for the page. A tutorial teaches through a guided exercise with a visible result. A how-to guide helps a practiced reader finish a task, with prerequisites and conditional steps. A reference page follows the structure of the settings or interfaces readers look up. An explanation answers a bounded question about how or why something works. Link to a separate page when readers need another form. [Diátaxis](https://diataxis.fr/start-here/)

## The same feature for four readers

**Proposal for the product lead**

> Approve a pilot so the finance team can save its export filters. We observed three analysts repeat the same filter selection across five exports. We could count how often they reuse a saved filter.

**Research finding for the product team**

> **Conclusion** Filter selection was a recurring step for the finance analysts we observed.
>
> **Evidence** Three finance analysts completed five exports. In all five, they selected the same filters again.
>
> **What to check** Do other teams repeat this setup often enough to warrant a wider pilot?

**Architecture explanation for a new engineer**

> The export worker reads the saved filter when the job starts. If it finds the filter, it applies it to the export query. If the filter was deleted, the worker stops the job.

**Status for the person waiting on the pilot**

> The pilot is ready on staging. Production access is waiting for approval.

## How to organize mixed material

When notes interleave two problems, give each problem a heading the reader would recognize. Keep its first check and follow-up actions together. This applies the guidance on organizing tasks by how people use and find them in *Developing Quality Technical Information*, chapters 2 and 8 of the second edition. [IBM Press](https://www.informit.com/store/developing-quality-technical-information-a-handbook-9780131477490)

When a measurement supports a recommendation, identify the criterion that makes it matter. If a backup must be usable within an hour, a timed restore tells readers whether it meets that requirement. This applies the distinction between evidence and the reasoning that connects it to a claim in *The Craft of Research*, fifth edition, chapters 7 and 8. [University of Chicago Press](https://press.uchicago.edu/ucp/books/book/chicago/C/bo215874008.html)
