# Behavior contract: <component or API>

## Scope

- Original artifact and version:
- File hash or evidence-ledger reference:
- Interface and calling environment:
- Included behavior:
- Excluded behavior:
- Contract revision:

## Inputs and outputs

| Operation | Input conditions | Result | State change or side effect | Evidence IDs | Confidence |
|---|---|---|---|---|---|
| `<operation>` | `<conditions>` | `<result>` | `<effect>` | `E01` | observed / measured / inferred / unknown |

## State

| State item | Initial value | When it changes | When it is read | Evidence IDs | Confidence |
|---|---|---|---|---|---|
| `<item>` | `<value or unknown>` | `<condition>` | `<condition>` | `E02` | `<label>` |

## Ordering and callbacks

Describe the order visible to a caller. State which events are synchronous, deferred or unknown.

## Error results

| Condition | Result | State after the operation | Evidence IDs | Confidence |
|---|---|---|---|---|
| `<condition>` | `<value>` | `<state>` | `E03` | `<label>` |

## Software-only acceptance cases

| Case | Setup | Call or input | Expected observable result | Evidence IDs | Confidence |
|---|---|---|---|---|---|
| `<case>` | `<setup>` | `<input>` | `<result>` | `E04` | `<label>` |

## Unknowns

| Question | Why it matters | Suggested software probe |
|---|---|---|
| `<question>` | `<effect on compatibility>` | `<probe or none>` |

## Provenance

`E01` through `E04` refer to entries in the separate evidence ledger. The ledger is not part of
the source-blind packet. This contract contains behavior descriptions, not source code or
pseudocode.
