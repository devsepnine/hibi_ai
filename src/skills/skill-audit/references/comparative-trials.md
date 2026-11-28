# Designing comparative trials

## Choose cases with different correct outcomes

For a skill that exports selected records, a routine case exports two checked rows. Change one condition at a time. Move keyboard focus to a third row to test command scope. Remove access to one checked record to test the documented permission behavior. Ask for a critique of the exporter to test the boundary with a review skill.

Record the input, allowed resources and actions, expected owner, observable result and what counts as failure. Grade the resulting files or selected record IDs when available. Match a phrase only when the exact text is part of the contract.

## Run a fair comparison

Keep the model, prompt, fixture, tool access and surrounding instructions comparable. Give each run fresh context and its own copy of mutable data. Withhold the answer key and prior conclusions. Remove the target's automatic imports from the baseline as well as its entry point. If that cannot be isolated, describe the confound. Include a single neighboring skill or a combination as a comparator when the ownership question warrants it.

When both outputs export the right records, record a tie on correctness. Measure the extra cost if it changes the decision. When both fail, inspect the shared fixture and scoring rule before blaming the skill. A small sample can demonstrate a failure or a benefit on those inputs. State the number of cases and runs before making a broader reliability claim.

## Locate the change

For catalog audits, `python3 scripts/inventory.py /path/to/skills` runs from this skill's directory using the Python standard library. Its JSON lists examined files, visible directories missing an entry point and groups of matching bodies. It ignores frontmatter, outer whitespace and paired Korean mirrors when matching. Exit code 1 means the catalog has no skill entries. Exit code 2 identifies a read failure. Correct the path or access and repeat the inventory before reporting coverage. Check translations separately for changed behavior. Differently worded instructions, parts of a file, code and assets still need comparison.

- A missing entry in the actual catalog points to packaging or discovery. A visible skill invoked explicitly that succeeds, coupled with an observed miss on an ordinary request, points to selection. Inspect the description and competing instructions.
- An invoked skill that reads a stale example or lacks a needed reference calls for tracing the resource path and resulting artifact. An unavailable service leaves the affected behavior unverified. Exercise any documented offline behavior separately.
- When two skills seem interchangeable, give them the same input and compare the requested output, side effects, unique exceptions and independently installed use. A shared paragraph can be intentional. A merge needs a surviving owner for every supported path.

For a repeated audit, keep the original failing fixture and add a new case after editing. Record enough of each run for a reader to see which observations led to a change and which checks remain open.
