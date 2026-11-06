# <Scope> QA Handoff: <YYYY-MM-DD>

## Overall summary

- **What changed**: <one sentence a non-developer can repeat in a standup>
- **Affected areas**: <screens / features / platforms>
- **Test focus**: <the one or two things most worth checking>

## Changes from previous behavior

| Area | Before | After |
|---|---|---|
| <screen or feature> | <previous behavior> | <current behavior> |

<!-- Omit this section if this work does not change or consume a changed API contract. -->
## API changes

API: `<endpoint>` · Contract: <specification or backend ticket link>

| Changed field | What changed | Screen or behavior using it |
|---|---|---|
| `<field>` | <plain description of the new or changed value> | <what the product does with this value> |

## QA checklist

- [ ] **1. <behavior to check>**
  - **Setup**: <only the account, data, or environment needed to reproduce>
  - **Reproduction steps**: 1. <action> 2. <action>
  - **Expected result**: <what QA should see or be able to do>

## Regression watch

| Existing feature | What to recheck |
|---|---|
| <feature that could be affected> | <behavior that should remain unchanged> |

## Automated verification results

- **Verified behavior**: <what was exercised and what happened>
- **Results**: <test counts and coverage, with the scope of each number>
- **Still to check**: <what sample responses or local execution did not verify>

## Glossary

| Term | Means |
|---|---|
| <term needed to understand this document> | <one short sentence that distinguishes it from similar terms> |

## Developer follow-up

- **<question to resolve>**: <who can answer and which QA check needs the answer>

<!-- Include this reference only when traceability is needed. Do not repeat it in QA items. -->
Reference: `<commit range or issue link>` · Prepared by: `<name>`
