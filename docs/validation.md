# Validation

## Validation

Validation is divided into four layers:

- **Schema:** file structure, identifier formats, required fields, and primitive
  value types.
- **Ontology:** declared entity types and properties, subject applicability,
  direction, value and target compatibility, cardinality, and permitted
  qualifiers.
- **Domain:** topic-specific rules and cross-entity invariants that do not
  belong in general storage validation.
- **Provenance:** required references on production structured data and
  well-formed, resolvable Markdown citations.

All production labels, descriptions, classifications, images, and statements
require at least one registered reference. The validator does not attempt to
distinguish externally derived values from other production data.

## Repository integrity

Validation checks all of the following:

- Every YAML mapping has exactly the fields permitted by the data model, with no
  unknown fields or duplicate keys.
- Every filename agrees exactly with the identifier inside the file:
  `entities/Q12.yaml` contains `id: Q12`, with equivalent rules for properties,
  references, and entity types.
- Every identifier has canonical syntax and is unique in its namespace.
- Every referenced entity, property, reference, and entity type exists.
- Statement identifiers are unique within their entity.
- Required maps and sequences are nonempty; an entity's required `statements`
  sequence may be empty.
- Every property is applicable to at least one type of its subject entity.
- Every entity-valued statement or qualifier targets an entity with at least
  one permitted target type.
- Statement value types agree with their property definitions.
- A `one` property occurs at most once among an entity's top-level statements.
- Every qualifier is allowed by its main property, is applicable to the
  statement's entity, and has the value type declared by the qualifier
  property. Cardinality is not applied to qualifiers.
- Typed scalar values, coordinates, URLs, dates, and timestamps have the
  representations and ranges defined by the data model.
- Every counter in `id_allocation.yaml` is positive and greater than all
  currently used identifiers in its namespace.
- Every `entity_context/Q<n>.md` filename names an existing entity.
- Every Markdown footnote label is a canonical, existing reference identifier,
  has one well-formed definition, and links exactly to
  `../references/R<n>.yaml`.

## Diagnostics

Validation collects all independent errors it can identify in one run.
Diagnostics contain the affected path, a YAML or Markdown line when available,
the relevant identifier when applicable, and a concise corrective message.
They are returned deterministically by path, location, identifier, and message.

The CLI exposes an operation of the form `<cli> validate <knowledge-base-path>`.
It exits successfully only when all enabled validation layers pass and otherwise
exits with a nonzero status.

## Initial implementation boundary

The initial library and CLI are read-only. Mutation and replacement behavior is
not specified by the MVP.
