# Spreadsheet-to-(Work)flow

Spreadsheet-to-flow is a CLI tool as part of a pipeline to generate a 
[FnO-Steps](https://spec.knows.idlab.ugent.be/fno-steps/latest/) workflow
skeleton from a description of *steps* and *states* in a spreadsheet.
In the generated flow a *state* has a corresponding *shape* which
checks for a default (also generated) SHACL *predicate path* in the data.
These generated shapes need to be customized to reflect the actual conditions.

```mermaid
---
title: A Pipeline to generate a FnO-Steps skeleton.
---
flowchart LR
    subgraph Spreadsheet-to-flow
    A[/ODS file/] --> stf[Spreadsheet-to-flow]
    stf --> yf[/mapping.yarrrml.yaml/]
    stf --> sf[/steps.json/]
    stf --> st[/states.csv/]
    stf --> sh[/shapes.csv/]
    end
    yf --> yp[YARRRML Parser]
    yp --> rmlf[/mapping.rml.ttl/]

    sf --> rmlm[RML Mapper]
    st --> rmlm
    sh --> rmlm
    rmlf --> rmlm

    rmlm --> steps[/steps.ttl/]
    rmlm --> states[/states.ttl/]
    rmlm --> shapes[/shapes.ttl/]
```

Spreadsheet-to-flow takes an ODS file as input and generates the files
`steps.json`, `states.csv`, `shapes.csv` and a corresponding
[YARRRML](https://rml.io/yarrrml/) mapping file `mapping.yarrrml.yaml`.

The generated YARRRML mapping file contains some default mappings to
generate a working FnO-Steps workflow. This mapping file can be modified
to meet the use case's needs.

The YARRRML file can then be converted to an [RML](https://rml.io/specs/rml/)
mapping file with [YARRRML Parser](https://github.com/rmlio/yarrrml-parser).

An RML mapping engine such as [RMLMapper](https://github.com/RMLio/rmlmapper-java)
then takes the RML mapping file together with the generated data files as input
and generated an FnO-Steps flow as output, consisting of
`steps.ttl`, `states.ttl` and `shapes.ttl`

## Running the complete pipeline with Docker / Podman
A Dockerfile which contains Spreadsheet-to-flow, YARRRML Parser and RMLMapper is provided.
It is tested on [Docker](https://www.docker.com/) and [Podman](https://podman.io/).

Build the container (`docker` can be replaced with `podman`):

```shell
docker build -t stf .
```

Run the container (this example generates the flow of `test-resources/testflow.ods`):

```shell
cd test-resources
docker container run -v "$(pwd)":/mnt/data --rm stf testflow.ods
```

All generated files will appear in `test-resources/testflow_output`.


## Building & Running only the Spreadsheet-to-flow CLI tool

To install the tool, install [Rust](https://www.rust-lang.org/tools/install),
then run

```shell
cargo install --path .
```

To run:

```shell
spreadsheet-to-flow path/to/spreadsheet.ods
```
