*   **Source:** PostgreSQL Extension Network
*   **RFC**: TBD
*   **Version**: 2.0.0-draft1
*   **Status**: Draft
*   **Category**: Standards Track
*   **Published**: TBD
*   **Authors**:
    *   David E. Wheeler, PGXN, Tembo

# RFC TBD #
# PostgreSQL Extension Network Distribution Metadata Specification #

## Abstract ##

This document describes version 2.0.0 of the [PGXN] source distribution
metadata specification, also known as the "PGXN Meta Spec." PGXN Metadata
ships with PGXN source distribution archives, and serves to describe the their
contents for the benefit of automated indexing, distribution, discovery,
full-text search, binary packaging, and more.

## Status of This Memo ##

This is a Standards Track document.

This RFC represents the consensus of the global community of PostgreSQL
extension developers, distributors, and packagers, generally referred to as
the "PostgreSQL Extension Ecosystem". It is formatted using the [Github
Flavored Markdown] variant of [Markdown], and the canonical copy will always
be found at [master.pgxn.org/meta/spec.txt]. A generated HTML-formatted copy
found at [pgxn.org/spec/] **MAY** also be considered canonical.

Information about the current status of this document, any errata, and how to
provide feedback on it **MAY** be obtained from its [source code repository].

## Copyright Notice ##

Copyright (c) 2010-2024 PGXN and the persons identified as the document
authors. All rights reserved.

This RFC is distributed under the [CC BY-SA 4.0] license.

Code Components extracted from this document **MUST** include the [PostgreSQL
License].

## Introduction ##

Distribution metadata describe important properties of source code [archive
files][archive file] distributed on the PostgreSQL Extension Network, or
[PGXN]. Tools that build PGXN source distribution archives **MUST** create a
metadata file in accordance with this specification and include it in the
distribution archive for use by automated tools that index, examine, package,
or install PGXN source distributions.

### Terminology ###

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [IETF RFC 2119].

This document makes use of the following additional terms:

#### Package ####

A collection of extensions that are released, versioned, and distributed
together.

#### Source Distribution ####

An [archive file] of the source code for the release of a [Package](#package),
together with metadata defined by this spec, distributed for other developers
to build, install, and use. The file name of a Source Distribution consists of
the package name, a dash, and the version, e.g., `pgtap-1.14.3.zip`.

Usually referred to as a "Distribution", including in this document. The full
term "Source Distribution" is used where necessary to distinguish from binary
distributions of a [Package](#package) (not covered by this RFC).

#### Extension ####

A software component that extends the capabilities of a PostgreSQL database or
cluster. Extensions **MAY** be `CREATE EXTENSION` [extensions], [background
workers], command-line apps, [loadable modules], shared libraries, and more.

#### Consumer ####

Code that reads a metadata file, deserializes it into a data structure in
memory, or interprets its elements.

#### Producer ####

Code that constructs a metadata data structure, serializes it into a byte
stream, and/or writes it to disk.

### Data Types ###

Properties in the [Structure](#structure) section describe data elements, each
of which has an associated data type as described herein. Each is based on the
primitive types defined by [JSON]: *object*, *array*, *string*, *number*, and
*boolean*. Other types are subtypes of these primitives and define compound
data structures or define constraints on the values of a data element.

#### Boolean ####

A *Boolean* is used to provide a true or false value. It **MUST** be
represented as a defined (not `null`) value.

#### String ####

A *String* is data element containing a non-zero length sequence of Unicode
characters.

#### Array ####

An *Array* is an ordered collection of one or more data elements. Items in an
Array **MAY** be of mixed types.

#### Object ####

An **Object** is an unordered set of key/value pairs, or "properties".
Property values are indexed by their corresponding [String](#string) keys. An
Object's values **MAY** be of mixed types.

#### Term ####

A *Term* is a [String](#string) that **MUST** be at least two characters long,
and contain no slash (`/`), backslash (`\`), control, or space characters.

#### Tag ####

A *Tag* is a [String](#string) that **MUST** be at least two and no more than
255 characters long, and contain no slash (`/`), backslash (`\`), or control
characters.

#### URI ####

*URI* is a [String](#string) containing a valid Uniform Resource Identifier or
Locator as defined by [IETF RFC 3986].

#### Path ####

*Path* is a [String](#string) with a relative file path that identifies a file
in the [Distribution](#source-distribution). The path **MUST** be specified
with unix conventions.

#### Version ####

A *Version* is a [String](#string) containing a value that describes the
version number of extensions or distributions, and adhere to the format of the
[Semantic Versioning 2.0.0 Specification][semver] with the exception of [Build
metadata], which is reserved for use by downstream packaging systems.

#### Version Range ####

A *Version Range* is a [String](#string) that describes a range of
[Versions](#version) that **MAY** be present or installed to fulfill
dependencies.

The simplest format for a Version Range is just the [Versions](#version)
itself, e.g. `2.4.2`. This means that **at least** version 2.4.2 must be
present. Versions may also be truncated to their major or minor parts, as
appropriate. For example, `2.4` means that **at least** version 2.4.0 must be
present.

Alternatively, a version range **may** use the operators `<` (less than), `<=`
(less than or equal), `>` (greater than), `>=` (greater than or equal), `==`
(equal), and `!=` (not equal). For example, the specification `< 2` means that
any version less than version 2 is suitable.

For more complicated situations, version specifications **may** be AND-ed
together using commas. The specification `>= 1.2, != 1.5.2, < 2.0`
indicates a version that must be **at least** 1.2, **less than** 2.0, and
**not equal to** 1.5.2.

#### License String ####

A *License String* is a [String](#string) identifying a licenses. Its values
are restricted to and defined by the [SPDX License List]. Examples:

*   `PostgreSQL`
*   `MIT`
*   `Apache-2.0`
*   `BSD-2-Clause`
*   `GPL-2.0-only`

#### License Expression ####

A *License Expression* is a [String](#string) that represents one or more
licenses by combining [License Strings](#license-string) into a single value.
The format is defined by [SPDX Standard License Expression]. Examples:

*   `LGPL-2.1-only OR MIT`
*   `LGPL-2.1-only AND MIT AND BSD-2-Clause`
*   `GPL-2.0-or-later WITH Bison-exception-2.2`
*   `LGPL-2.1-only OR BSD-3-Clause AND MIT`

#### purl ####

A *purl* is a [String](#string) containing a valid package in the format
defined by the [purl spec]. All known [purl Types] **MAY** be used, as well as
`pgxn` for PGXN packages and `postgres` for PostgreSQL core [contrib] or
development packages. Some examples:

*   `pkg:pgxn/pgtap`
*   `pkg:postgres:pg_regress`
*   `pkg:generic/python3@3`
*   `pkg:pypi/pyarrow@11.0.0`

The version part of a purl, when present, will be interpreted as a [Version
Range](#version-range), allowing for flexible specification of supported
versions where necessary. Note that versions must be [percent-encoded]. For
example, this purl encodes `>=1.2,!=1.5.2,< 2.0`:

*   `pkg:pgxn/pgmq@%3E%3D1.2%2C+%21%3D1.5.2%2C%3C+2.0`

#### Platform ####

A *Platform* is a [String](#string) that identifies a computing platform as a
one to three dash-delimited substrings: An OS name, the OS version, and the
architecture: `$os-$version-$architecture`.

If the string contains no dash, it represents only the OS. If it contains a
single dash, the values represent the OS and the architecture. The complete
list of values will be determined by the [bulid farm animals]. Some likely
Examples:

*   `linux`: Any Linux
*   `linux-amd64`: Any Linux on amd64/x86_64
*   `gnulinux-amd64`: [GNU] Linux on amd64/x86_64
*   `musllinux-1.2-arm64`: [musl] Linux v1.2 on arm64/aarch64
*   `darwin`: Any Darwin (macOS)
*   `darwin-23.5.0-arm64`: Darwin (macOS) 23.5.0 on arm64/aarch64

## Structure

The metadata structure is an [Object](#object). This section describes valid
properties of the [Object](#object).

Any properties not described in this specification document (whether top-level
or within [Objects](#object) described herein) are considered *custom
properties* and **MUST** begin with an "x" or "X" and be followed by an
underscore; i.e., they **MUST** match the regular expression pattern
`^[xX]_.`. If a custom property refers to an [Object](#object), properties
within it do not need an "x_" or "X_" prefix.

Metadata [Consumers](#consumer) **MAY** ignore any or all custom properties.
All other properties not described herein are invalid and **SHOULD** be
ignored by [Consumers](#consumer). [Producers](#producer) **MUST NOT**
generate or output invalid properties.

For each property, one or more examples are provided followed by a
description. The description begins with the version of spec in which the
property was added or in which the definition was modified, whether the
property is **REQUIRED** or **OPTIONAL**, and the data type of the value.
These items are in parentheses, brackets, and braces, respectively.

If a data type is an [Object](#object), valid sub-properties will be described
as well.

All examples are represented as [JSON].

<!-- Nothing deprecated yet.
Some properties are marked *Deprecated*. These are shown for historical
context and **MUST NOT** be produced in or consumed from any metadata structure of
version 2 or higher.
-->

### Required Properties

#### abstract ####

``` json
"abstract": "Unit testing for PostgreSQL"
```

(Spec 1) [required] {[String](#string)}

This is a short description of the purpose of the [Package](#package) provided
by the [Distribution](#source-distribution).

#### maintainers ####

```json
"maintainers": [
  {
    "name": "David E. Wheeler",
    "url": "https://pgxn.org/user/theory"
  }
]
```

```json
"maintainers": [
  {
    "name": "David E. Wheeler",
    "email": "theory@pgxn.org",
    "url": "https://pgxn.org/user/theory"
  },
  {
    "name": "Josh Berkus",
    "email": "jberkus@pgxn.org"
  }
]
```

(Spec 2) [required] {[Array](#array) of [Objects]{#object}}

This property indicates the person(s) to contact concerning the
[Distribution](#source-distribution). Each [Object](#object) in the
[Array](#array) consists of the following properties:

*   **name**: The name of the maintainer. **REQUIRED**.
*   **email**: The email address of the maintainer.
*   **url**: The URL for the maintainer.

Either `email` or `url` or both **MUST** be present.

This property provides a general contact list independent of other structured
fields provided within the [resources](#resources) field, such as `issues`.
The addressee(s) can be contacted for any purpose including but not limited
to: (security) problems with the [Distribution](#source-distribution),
questions about the [Distribution](#source-distribution), or bugs in the
[Distribution](#source-distribution).

A [Distribution](#source-distribution)'s original author is usually the
contact listed within this field. Co-maintainers, successor maintainers, or
mailing lists devoted to the distribution **MAY** also be listed in addition
to or instead of the original author.

#### license ####

``` json
"license": "MIT"
```

``` json
"license": "MIT AND BSD-2-Clause"
```

(Spec 1) [required] {[License String](#license-string) or [License Expression](#license-expression)}

One or more licenses that apply to some or all of the files in the
[Distribution](#source-distribution). For [License
Expressions](#license-expression), the [Distribution](#source-distribution)
documentation **SHOULD** be consulted to clarify the interpretation of
multiple licenses.

#### contents ####

``` json
"contents": {
  "extensions": {
    "pair": {
      "sql": "sql/pair.sql",
      "doc": "doc/pair.md",
      "abstract": "A key/value pair data type",
      "preload": "session",
      "tle": true,
      "control": "pair.control"
    }
  }
```

``` json
"contents": {
  "workers": {
    "pair_pruner": {
      "bin": "bin/pair_pruner",
      "doc": "doc/pair_pruner.md",
      "abstract": "A worker to periodically prune pairs"
    }
  },
  "modules": {
    "lib_pair": {
      "lib": "lib/lib_pair",
      "doc": "doc/lib_pair.md",
      "abstract": "A library hooking function calls to convert pairs to named parameters",
      "load": "shared_preload_libraries"
    }
  }
}
```

``` json
"contents": {
  "apps": {
    "pair_rand": {
      "bin": "bin/pair_rand",
      "doc": "doc/pair_rand.md",
      "abstract": "Command to generate random pairs of strings"
    }
  },
  "libraries": {
    "ruby_pair": {
      "dir": "lib/gems",
      "abstract": "Ruby libraries required to run the extension"
    }
  }
}
```

(Spec 1) [required] {[Object](#object) of [Objects](#object) of [Terms](#term)}

A description of what's included in the [Package](#package) provided by the
[Distribution](#source-distribution). This information is used by [PGXN] to
build indexes identifying in which [Package](#package) various
[Extensions](#extension) can be found.

The properties of `contents` identify the types of [Extensions](#extension) in
the [Distribution](#source-distribution). At least one property **MUST** be
present in the `contents` object. The properties are as follows:

*   **extensions**: [Object](#object) describing `CREATE EXTENSION`
    [extensions]. Properties are extension name [Terms](#term) pointing to
    [Objects](#object) with the following fields:
    *   **sql**: A [Path](#path) pointing to the SQL file used by `CREATE
        EXTENSION`. **REQUIRED**.
    *   **control**: A [Path](#path) pointing to the [control file] used by
        `CREATE EXTENSION`. **REQUIRED**.
    *   **doc**: A [Path](#path) pointing to the main documentation file for
        the extension, which **SHOULD** be more than a README.
    *   **abstract**: A [String](#string) containing a short description of
        the extension.
    *   **tle**: A [Boolean](#boolean) that, when `true`, indicates that the
        extension can be used as a [trusted language extension].
*   **workers**: [Object](#object) describing [background workers]. Properties
    are worker name [Terms](#term) pointing to [Objects](#object) with the
    following properties:
    *   **src**: A [Path](#path) pointing to the main source file for the
        background worker. **REQUIRED**.
    *   **doc**: A [Path](#path) pointing to the main documentation file for
        the background worker, which **SHOULD** be more than a README.
    *   **abstract**: A [String](#string) containing a short description of
        the background worker.
*   **apps**: [Objects](#object) describing applications, command-line or
    otherwise. Properties are are app name [Terms](#term) pointing to
    [Objects](#object) with the following properties:
    *   **src**: A [Path](#path) pointing to the main source file for the app.
        **REQUIRED**.
    *   **doc**: A [Path](#path) pointing to the main documentation file for
        the app, which **SHOULD** be more than a README.
    *   **abstract**: A [String](#string) containing a short description of
        the app.
*   **modules**: [Objects](#object) describing [loadable modules] that can be
    loaded into Postgres. Properties are module name [Terms](#term) pointing
    to [Objects](#object) with the following properties:
    *   **src**: A [Path](#path) pointing to the main source file for the
        module. **REQUIRED**.
    *   **doc**: A [Path](#path) pointing to the main documentation file for
        the module, which **SHOULD** be more than a README.
    *   **abstract**: A [String](#string) containing a short description of
        the module.
    *   **preload**: A [String](#string) that indicates that the extension's
        libraries **MAY** be loaded in advance. Its three possible values are:
        `shared`, `session`, and `local`. Extensions that require early or
        late loading of their module **MAY** optionally append a space and
        then either `early` or `late`.
*   **libraries**: [Objects](#object) listing other libraries that **MAY**
    ship in the package and need to be installed but are not [loadable
    modules], such as a dynamic library used by an app. Properties are library
    name [Terms](#term) pointing to [Objects](#object) with the following
    properties:
    *   **src**: A [Path](#path) pointing to the main source file or directory
       of files for the library. **REQUIRED**.
    *   **doc**: A [Path](#path) pointing to the main documentation file for
        the library, which **SHOULD** be more than a README.
    *   **abstract**: A [String](#string) containing a short description of
        the app.

#### meta-spec ####

``` json
"meta-spec": {
  "version": "2.0.0",
  "url": "https://pgxn.org/meta/v2/spec.txt"
}
```

(Spec 1) [required] {[Map](#Map)}

This field indicates the [Version](#Version) of the PGXN Meta Spec that
**SHOULD** be used to interpret the metadata. Consumers **MUST** check this
key as soon as possible and abort further metadata processing if the meta-spec
[Version](#version) is not supported by the consumer.

The following properties are valid, but only `version` is **REQUIRED**.

*   **version**: The [Version](#version) of the PGXN Meta Spec against which
    the metadata object was generated.

*   **url**: The [URI](#uri) of the metadata specification corresponding to
    the given version. This is strictly for human-consumption and **SHOULD
    NOT** impact the interpretation of the metadata structure.

#### name ####

Example:

``` json
"name": "pgTAP"
```

(Spec 1) [required] {[Term](#Term)}

This property is the name of the [Package](#package) provided by the
[Distribution](#source-distribution). This is usually the same as the name of
the "main extension" in the [contents](#contents) of the [Package](#package),
but **MAY** be completely unrelated. This value will be used in the
[Distribution](#source-distribution) file name on [PGXN].

#### version ####

``` json
"version": "1.3.6"
```

(Spec 1) [required] {[Version](#version)}

This property gives the version of the [Distribution](#source-distribution) to
which the metadata structure refers. Its value **MUST** be a
[Version](#version).

All of the items listed in [contents](#contents) will be considered to have
this version; any references they make to a version, such as the [control
file], **SHOULD** be compatible with this version.

### Optional Fields ###

#### description ####

``` json
"description": "pgTAP is a suite of database functions that make it easy to write TAP-emitting unit tests in psql scripts or xUnit-style test functions."
```

(Spec 1) [optional] {[String](#string)}

A longer, more complete description of the purpose or intended use of the
[Package](#package) provided by the [Distribution](#source-distribution),
answering the question "what is this thing and what value is it?"

#### generated_by ####

Example:

``` json
"generated_by": "Module::Build::PGXN version 0.42"
```

(Spec 1) [optional] {[String](#string)}

This property indicates the tool that was used to create this metadata. There
are no defined semantics for this property, but it is traditional to use a
string in the form "Software package version 1.23", or the maintainer's name
if the metadata was generated by hand.

#### classification ####

``` json
{
  "tags": [
    "testing",
    "pair",
    "parameter"
  ],
  "categories": [
    "Machine Learning"
  ]
}
```

(Spec 2) [optional] {[Object](#object) of [Arrays](#array) of [Tags](#tag)}

Classification metadata associates additional information about the
[Package](#package) provided by the [Distribution](#source-distribution) to
improve discovery. This [Object](#object) **MUST** contain at least one of the
following properties:

*   **tags**: An [Array](#array) of one or more keyword [Tags](#tag)s that
    describe the distribution.
*   **categories**: An [Array](#array) of at least one and no more than three
    of the following [Strings](#string) that categorize the distribution:
    *   Analytics
    *   Auditing and Logging
    *   Change Data Capture
    *   Connectors
    *   Data and Transformations
    *   Debugging
    *   Index and Table Optimizations
    *   Machine Learning
    *   Metrics
    *   Orchestration
    *   Procedural Languages
    *   Query Optimizations
    *   Search
    *   Security
    *   Tooling and Admin

#### ignore ####

``` json
"ignore": [
  "/src/private",
  "/src/file.sql",
  "*.html"
]
```

(Spec 2) [optional] {[Array](#Array) of [Strings](#string)}

This [Array](#array) describes any files or directories that are private to
the [Distribution](#source-distribution) and **SHOULD** be ignored by indexing
or search tools. Values are [Paths](#path) or [Strings](#string) based on a
subset of the [gitignore format].

#### dependencies ####

``` json
"dependencies": {
  "postgres": {
    "version": "14"
  },
}
```

``` json
"dependencies": {
  "postgres": {
    "version": ">= 12, < 17",
    "with": [ "xml", "uuid", "perl" ]
  },
  "pipeline": "pgxs",
  "packages": {
    "build": {
      "requires": [
        "pkg:generic/awk",
        "pkg:generic/perl@5.20"
      ],
      "recommends": [
        "pkg:generic/jq",
        "pkg:generic/perl@5.40"
      ]
    }
  }
}
```

``` json
"dependencies": {
  "pipeline": "pgrx",
  "platforms": [
    "linux-amd64",
    "linux-amd64v3",
    "gnulinux-arm64",
    "musllinux-amd64",
    "darwin-23.5.0-arm64"
  ],
  "packages": {
    "configure": {
      "requires": [ "pkg:cargo/cargo-pgrx@%3D0.11.4" ]
    },
    "test": {
      "requires": [
        "pkg:postgres/pg_regress",
        "pkg:postgres/plpgsql"
        "pkg:pgxn/theory/pgtap@1.1.0"
      ]
    },
    "run": {
      "requires": [
        "pkg:postgres/plperl"
        "pkg:pgxn/theory/hostname"
      ]
    }
  }
}
```

``` json
"dependencies": {
  "postgres": {
    "version": ">= 15, < 16"
  },
  "pipeline": "pgxs",
  "platforms": [
    "linux-amd64", "linux-arm64",
    "darwin-amd64", "darwin-arm64"
  ],
  "packages": {
    "configure": {
      "requires": {
        "external": [
          "pkg:cargo/cargo-pgrx@%3D0.11.4",
          "pkg:generic/bison",
          "pkg:generic/cmake",
          "pkg:generic/flex",
          "pkg:generic/readline",
          "pkg:generic/openssl",
          "pkg:generic/pkg-config"
        ]
      }
    },
    "run": {
      "requires": [
          "pkg:generic/penblas",
          "pkg:generic/python3",
          "pkg:generic/readline",
          "pkg:generic/openssl",
          "pkg:generic/bison"
      ],
      "recommends": [
          "pkg:pypi/pyarrow@11.0.0",
          "pkg:pypi/catboost",
          "pkg:pypi/lightgbm",
          "pkg:pypi/torch",
          "pkg:pypi/torchaudio",
          "pkg:pypi/torchvision",
          "pkg:pypi/xgboost",
          "pkg:pypi/accelerate",
          "pkg:pypi/bitsandbytes",
          "pkg:pypi/ctransformers",
          "pkg:pypi/huggingface-hub",
          "pkg:pypi/deepspeed",
          "pkg:pypi/einops",
          "pkg:pypi/optimum",
          "pkg:pypi/peft",
          "pkg:pypi/tokenizers",
          "pkg:pypi/transformers",
          "pkg:pypi/transformers-stream-generator",
          "pkg:pypi/InstructorEmbedding",
          "pkg:pypi/sentence-transformers",
          "pkg:pypi/rouge",
          "pkg:pypi/sacrebleu",
          "pkg:pypi/sacremoses",
          "pkg:pypi/datasets",
          "pkg:pypi/orjson",
          "pkg:pypi/langchain"
      ]
    }
  },
  "variations": [
    {
      "where": {
        "platforms": ["linux"]
      },
      "dependencies": {
        "packages": {
          "run": {
            "recommends": [
              "pkg:pypi/auto-gptq",
              "pkg:pypi/xformers"
            ]
          }
        }
      }
    }
  ]
},
```

(Spec 2) [optional] {[Object](#object)}

This property identifies dependencies required to configure, build, test,
install, and run the [Package](#package) provided by the
[Distribution](#source-distribution), expressed as [purls](#purl). These
include not only other extensions, but also external libraries, system
dependencies, and versions of PostgreSQL --- as well as any OS and
architectures ([arm64], [amd64], etc.).

[Consumers](#consumer) **SHOULD** use this data to determine what dependencies
to install.

Properties:

*   **platforms**: An [Array](#array) of one or more [Platform](#platform)
    strings that identify OSes and architectures supported by the
    [Package](#package) provided by the [Distribution](#source-distribution).
    If this property is not present, [Consumers](#consumer) **SHOULD** assume
    that the [Package](#package) supports any platform that PostgreSQL
    supports. This property is typically needed only when the
    [Package](#package) depends on platform-specific features.

*   **postgres**: An [Object](#object) describing the versions of PostgreSQL
    required by the [Package](#package) provided by the
    [Distribution](#source-distribution). The object supports the following
    properties:
    *   **version**: A [Version Range](#version-range) identifying the
        supported versions of PostgreSQL. **REQUIRED**.
    *   **with**: An [Array](#array) of [Terms](#term) that correspond
        features that are required to be compiled into PostgreSQL. Each
        corresponds to the appropriate `--with` [configure flags].
        **OPTIONAL**.

*   **pipeline**: A [Term](#term) identifying the build pipeline required to
    configure, build, test, and install the [Package](#package) provided by
    the [Distribution](#source-distribution). Supported values
    **MAY** include:

    *   pgxs
    *   meson
    *   pgrx
    *   autoconf
    *   gem
    *   cpan
    *   pip
    *   go
    *   rust

    If this field is not present, [Consumers](#consumer) **MAY** use
    heuristics to ascertain the pipeline to use, such as the presence or
    absence of a `configure.sh`, `Makefile`, or `Cargo.toml` file.

*   **packages**: An [Object](#object) defining dependencies required for
    different phases of the build process. The supported property names are
    `configure`, `build`, `test`, `run`, and `develop`. Values are
    [Objects](#object) with at least one of the properties `requires`,
    `recommends`, `suggests`, and `conflicts` pointing to [Arrays](#array) of
    [purls](#purl) that identify the packages.

    See the [Package Spec](#packages-spec) for the full definition of this
    property.

*   **variations**: An [Array](#array) of [Object](#object)s that define
    dependency variations. Each object contains two properties:
    *   **where**: An [Object](#object) containing the subset of the
        [dependencies](#dependencies) to identify a variation, such as
        `{ "platforms": ["gnulinux-arm64", "gnulinux-amd64"] }` for Linux
        configurations, or `{"postgres": { "version": ">= 16, < 17" }}` for
        PostgreSQL versions. **MUST NOT** include a `variations` property.
    *   **dependencies**: An [Object](#object) containing the subset of
        [dependencies](#dependencies) required for the `where` property's
        configuration. **MUST NOT** include a `variations` property.

#### resources ####

``` json
"resources": {
  "homepage": "https://pair.example.com",
  "issues": "https://github.com/example/pair/issues",
  "docs": "https://pair.example.com/docs",
  "support": "https://github.com/example/pair/discussions",
  "repository": "https://github.com/example/pair",
  "badges": [
    {
      "alt": "Test Status",
      "src": "https://test.packages.postgresql.org/github.com/example/pair.svg"
    }
  ]
}
```

(Spec 2) [optional] {[Object](#object)}

This property provides external information about the [Package](#package)
provided by the [Distribution](#source-distribution). [Consumers](#consumer)
**MAY** use this data for links and displaying useful information about the
package.

The `resources` object **MUST** contain at least one of the following
properties:

*   **homepage**: [URI](#uri) for the official home of the project on the web.
*   **issues**: [URI](#uri) for the package's issue tracking system.
*   **repository**: [URI](#uri) for the package's source code repository.
*   **docs**: [URI](#uri) for the package's documentation.
*   **support**: [URI](#uri) for support resources and contacts for the
    package.
*   **badges**: An [Array](#array) of [Objects](#object) linking to badge
    images that **SHOULD** follow the [Shields badge specification]. It
    **MUST** have at least one entry, and all entries require two properties:
    *   **src**: The [URI](#uri) for the badge.
    *   **alt**: Alt text for the badge.

#### artifacts ####

``` json
[
  {
    "type": "source",
    "url": "https://github.com/theory/pg-pair/releases/download/v1.1.0/pair-1.1.0.zip",
    "sha256": "2b9d2416096d2930be51e5332b70bcd97846947777a93e4a3d65fe1b5fd7b004"
  },
  {
    "type": "binary",
    "url": "https://github.com/theory/pg-pair/releases/download/v1.1.0/pair-1.1.0-linux-amd64.tar.gz",
    "sha1": "12d9bc5cfb6bc3c453627eac69511f48be63cfc0"
  },
  {
    "type": "binary",
    "url": "https://github.com/theory/pg-pair/releases/download/v1.1.0/pair-1.1.0-linux-arm64.tar.gz",
    "sha1": "787dc39137f7d1510a33ab0a1b8905cd5f3f72d1"
  }
]
```

(Spec 2) [optional] {[Array](#array)}

An [Array](#array) of [Objects](#objects) describing links and checksums for
downloading the [Package](#package) provided by the
[Distribution](#source-distribution) in one or more formats, including source
code, binaries, system packages, and more. [Consumers](#consumer) my use this
information to determine the best option for installing an extension on a
particular system. Useful for projects that publish their own binaries, such
as in GitHub releases.

The [Array](#array) **MUST** have at least one [Object](#object). The
properties of each [Object](#object) are:

*   **url**: A [URI](#uri) to download the artifact. **REQUIRED**.
*   **sha256** or **sha512**: A [String](#string) containing a SHA-256 or
    SHA-512 checksum in hex format. **REQUIRED**.
*   **type**: The type of artifact. **MUST** be a single lowercase word
    describing the artifact, such as none of `binary`, `source`, `rpm`,
    `homebrew`, etc. **REQUIRED**.
*   **platform**: A [Platform](#platform) string identifying the platform the
    artifact was built for. **RECOMMENDED** for packages compiled for a
    specific platform, such as a C [Extension](#extension) compiled for
    `linux-arm64`.

Each URL **MUST** properly resolve and the checksum **MUST** match.

## Packages Spec ##

The `packages` sub-property of the [dependencies](#dependencies) property
defines the relationship between a [Distribution](#source-distribution) and
external dependencies --- including other PGXN [Packages](#package), system
packages, and third-party packages --- expressed as [purls](#purl). The
structure is an [Object](#object) that specifies package dependencies into
*Phases* of activity in the installation process, and *Relationships* that
indicate how dependencies **SHOULD** be resolved.

For example, to specify that the [PGXN] extension `pgtap` by user `theory` is
required during the `test` phase, this entry would appear in the
[Distribution](#source-distribution) metadata:

``` json
"packages": {
  "test": {
    "requires": [ "pkg:pgxn/theory/pgtap" ]
  }
}
```

All known [purl Types] **MAY** be used to identify dependencies and specific
versions. [Producers](#producer) **SHOULD** specify dependencies of two
additional types as appropriate:

*   **`pkg:pgxn`**: Packages distributed via [PGXN]. These **MUST** include
    both the username and package name, e.g., `pkg:pgxn/theory/pair`.
*   **`pkg:postgres`**: Dependencies distributed as part of the PostgreSQL
    core, including [contrib] or development packages such as [auto_explain],
    [dblink], [pg_regress] and [pg_isolation_regress]. Example:
    `pkg:postgres/dblink`.

[Producers](#producer) **SHOULD** avoid OS-specific [purls](#purl) such as
`pkg:rpm:/libreadline-dev` unless the package supports only OSes that provide
such packages. See the "variations" property of the
[dependencies](#dependencies) object for platform-specific dependency
specification.

[Consumers](#consumer) **SHOULD** use [Repology] to resolve `pkg:generic`
[purls](#purl) to packages specific to the platform on which an extension is
being built. This is useful for specifying system dependencies that vary by
name and packaging system. Otherwise, they **MAY** use whatever techniques or
heuristics are appropriate to install dependencies.

### Phases ###

Requirements for regular use **MUST** be listed in the `runtime` phase. Other
requirements **SHOULD** be listed in the earliest stage in which they are
required, and [Consumers](#consumer) **MUST** accumulate and satisfy
requirements across phases before executing the action. For example, `build`
requirements **MUST** also be available during the `test` phase.

  before action | requirements that **MUST** be met
----------------|-----------------------------------
  ./configure   | configure
  make          | configure, runtime, build
  make test     | configure, runtime, build, test

Consumers that install the [Package](#package) provided by the
[Distribution](#source-distribution) **MUST** ensure that *runtime*
requirements are also installed and **MAY** install dependencies from other
phases.

  after action  | requirements that **MUST** be met
----------------|-----------------------------------
  make install  | runtime

*   **configure**: The configure phase occurs before any dynamic configuration
    has been attempted. Dependencies required by the configure phase **MUST**
    be available for use before the build tool has been executed.

*   **build**: The build phase is when the
    [Distribution](#source-distribution)'s source code is compiled (if
    necessary) and otherwise made ready for installation.

*   **test**: The test phase is when the
    [Distribution](#source-distribution)'s automated test suite is run. Any
    dependency needed only for testing and not for subsequent use **SHOULD**
    be listed here.

*   **runtime**: The runtime phase refers not only to when the contents of the
    [Package](#package) provided by the [Distribution](#source-distribution)
    are installed, but also to its continued use. Any package that is a
    dependency for regular use of this [Package](#package) **SHOULD** be
    indicated here.

*   **develop**: The develop phase's packages are needed to work on the
    [Package](#package)'s source code as its maintainer does. These tools
    might be needed to build a release archive, to run maintainer-only tests,
    or to perform other tasks related to developing new versions of the
    [Package](#package).

### Relationships ###

*   **requires**: These dependencies **MUST** be installed for proper
    completion of the phase.

*   **recommends**: **RECOMMENDED** dependencies are *strongly* encouraged and
    **SHOULD** be satisfied except in resource constrained environments.

*   **suggests**: These dependencies are **OPTIONAL**, are suggested for
    enhanced operation of the described distribution, and **MAY** be
    satisfied.

*   **conflicts**: These dependencies **MUST NOT** be installed when the phase
    is in operation. This is a very rare situation, and the conflicts
    relationship **SHOULD** be used with great caution, or not at all.

Serialization
=============

Distribution metadata **SHOULD** be serialized as JSON-encoded data and
packaged with distributions as the file `META.json`.

See Also
========

* [CPAN Meta Spec]
* [PGXN]
* [JSON]
* [Semantic Versioning 2.0.0][semver]

Contributors
============

The PGXN Meta Spec was originally based on the [CPAN Meta Spec], which was
written by Ken Williams in 2003 and has since been updated by Randy Sims,
David Golden, Ricardo Signes, Adam Kennedy, and contributors.

  [PGXN]: https://pgxn.org "PostgreSQL Extension Network"
  [source code repository]: https://github.org/pgxn/pgxn-meta-spec
  [PostgreSQL License]: https://www.postgresql.org/about/licence/
  [CC BY-SA 4.0]: https://creativecommons.org/licenses/by-sa/4.0/ "Attribution-Sharealike 4.0 International"
  [Github Flavored Markdown]: https://github.github.com/gfm/
  [Markdown]: https://daringfireball.net/projects/markdown/
  [master.pgxn.org/meta/spec.txt]: https://master.pgxn.org/meta/spec.txt
  [pgxn.org/spec/]: https://pgxn.org/spec/
  [archive file]: https://en.wikipedia.org/wiki/Archive_file
    "Wikipedia: “Archive file”"
  [`semver`]: https://pgxn.org/dist/semver/
  [`vector`]: https://pgxn.org/dist/vector/
  [`citus`]: https://pgxn.org/dist/citus/
  [`CREATE EXTENSION` statement]: https://www.postgresql.org/docs/current/static/sql-createextension.html
  [IETF RFC 2119]: https://www.ietf.org/rfc/rfc2119.txt
  [JSON]: https://json.org/
  [IETF RFC 3986]: https://www.rfc-editor.org/info/rfc3986
    "RFC 3986: Uniform Resource Identifier (URI): Generic Syntax"
  [purl spec]: https://github.com/package-url/purl-spec/blob/master/PURL-SPECIFICATION.rst
    "package-url/purl-spec: A minimal specification a “mostly universal” package URL"
  [percent-encoded]: https://github.com/package-url/purl-spec/blob/master/PURL-SPECIFICATION.rst#character-encoding
    "Package URL specification: Character encoding"
  [purl Types]: https://github.com/package-url/purl-spec/blob/master/PURL-TYPES.rst
    "Package URL Type definitions"
  [semver]: https://semver.org/
  [SPDX License List]: https://github.com/spdx/license-list-data/
  [SPDX Standard License Expression]: https://spdx.github.io/spdx-spec/v3.0/annexes/SPDX-license-expressions/
  [control file]: https://www.postgresql.org/docs/current/extend-extensions.html
  [trusted language extension]: https://github.com/aws/pg_tle
    "pg_tle: Framework for building trusted language extensions for PostgreSQL"
  [background workers]: https://www.postgresql.org/docs/current/bgworker.html
    "PostgreSQL Docs: Background Worker Processes"
  [loadable modules]: https://www.postgresql.org/docs/16/gist-extensibility.html
  [gitignore format]: https://git-scm.com/docs/gitignore
  [bulid farm animals]: https://buildfarm.postgresql.org/cgi-bin/show_members.pl
  [configure flags]: https://www.postgresql.org/docs/current/install-make.html#CONFIGURE-OPTIONS-FEATURES
  [Repology API]: https://repology.org/api "Repology, the packaging hub: API"
  [contrib]: https://www.postgresql.org/docs/current/contrib.html
  [auto_explain]: https://www.postgresql.org/docs/current/auto-explain.html
  [dblink]: https://www.postgresql.org/docs/current/dblink.html
  [pg_regress]: https://github.com/postgres/postgres/tree/master/src/test/regress
  [pg_isolation_regress]: https://github.com/postgres/postgres/tree/master/src/test/isolation
  [Shields badge specification]: https://github.com/badges/shields/blob/master/spec/SPECIFICATION.md
  [CPAN Meta Spec]: https://metacpan.org/pod/CPAN::Meta::Spec
  [musl]: https://musl.libc.org/
  [GNU]: https://www.gnu.org/software/libc/
