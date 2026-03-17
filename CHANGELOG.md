# `content-tag` Changelog
## Release (2025-11-25)

content-tag 4.1.0 (minor)

#### :rocket: Enhancement
* `content-tag`
  * [#112](https://github.com/embroider-build/content-tag/pull/112) indentation stripping ([@NullVoxPopuli](https://github.com/NullVoxPopuli))
  * [#108](https://github.com/embroider-build/content-tag/pull/108) add utf16 code points range ([@patricklx](https://github.com/patricklx))

#### :memo: Documentation
* `content-tag`
  * [#111](https://github.com/embroider-build/content-tag/pull/111) Fix return type in readme to include code and map (as the dts already… ([@NullVoxPopuli](https://github.com/NullVoxPopuli))

#### :house: Internal
* `content-tag`
  * [#107](https://github.com/embroider-build/content-tag/pull/107) Configure prettier ([@NullVoxPopuli](https://github.com/NullVoxPopuli))
  * [#104](https://github.com/embroider-build/content-tag/pull/104) Add additional js/runtime tests for how to use character offsets ([@NullVoxPopuli](https://github.com/NullVoxPopuli))

#### Committers: 2
- Patrick Pircher ([@patricklx](https://github.com/patricklx))
- [@NullVoxPopuli](https://github.com/NullVoxPopuli)
## Release (2025-05-07)

content-tag 4.0.0 (major)

#### :boom: Breaking Change
* `content-tag`
  * [#101](https://github.com/embroider-build/content-tag/pull/101) parse API: make bytes vs characters explicit ([@ef4](https://github.com/ef4))

#### Committers: 1
- Edward Faulkner ([@ef4](https://github.com/ef4))
## Release (2025-05-01)

content-tag 3.1.3 (patch)

#### :bug: Bug Fix
* `content-tag`
  * [#99](https://github.com/embroider-build/content-tag/pull/99) content-tag accidentally strips "declare" keyword from class properties ([@ef4](https://github.com/ef4))

#### Committers: 1
- Edward Faulkner ([@ef4](https://github.com/ef4))
## Release (2025-03-20)

content-tag 3.1.2 (patch)

#### :bug: Bug Fix
* `content-tag`
  * [#97](https://github.com/embroider-build/content-tag/pull/97) Support automatic export default when using TS satisfies keyword ([@ef4](https://github.com/ef4))

#### Committers: 1
- Edward Faulkner ([@ef4](https://github.com/ef4))
## Release (2025-02-01)

content-tag 3.1.1 (patch)

#### :bug: Bug Fix
* `content-tag`
  * [#94](https://github.com/embroider-build/content-tag/pull/94) Replace random import alias with constant ([@davidtaylorhq](https://github.com/davidtaylorhq))

#### Committers: 1
- David Taylor ([@davidtaylorhq](https://github.com/davidtaylorhq))
## Release (2024-12-17)

content-tag 3.1.0 (minor)

#### :rocket: Enhancement
* `content-tag`
  * [#89](https://github.com/embroider-build/content-tag/pull/89) Allow direct importing of the standalone module ([@NullVoxPopuli](https://github.com/NullVoxPopuli))

#### Committers: 1
- [@NullVoxPopuli](https://github.com/NullVoxPopuli)
## Release (2024-11-08)

content-tag 3.0.0 (major)

#### :boom: Breaking Change
* `content-tag`
  * [#86](https://github.com/embroider-build/content-tag/pull/86) Change return type of process from string to { code, map } ([@NullVoxPopuli](https://github.com/NullVoxPopuli))

#### Committers: 1
- [@NullVoxPopuli](https://github.com/NullVoxPopuli)
## Release (2024-11-05)

content-tag 2.0.3 (patch)

#### :bug: Bug Fix
* `content-tag`
  * [#83](https://github.com/embroider-build/content-tag/pull/83) Strip hygiene code in favor of UUIDs ([@wagenet](https://github.com/wagenet))

#### :house: Internal
* `content-tag`
  * [#81](https://github.com/embroider-build/content-tag/pull/81) Update toolchain and some deps ([@wagenet](https://github.com/wagenet))

#### Committers: 1
- Peter Wagenet ([@wagenet](https://github.com/wagenet))
## Release (2024-09-21)

content-tag 2.0.2 (patch)

#### :bug: Bug Fix
* `content-tag`
  * [#79](https://github.com/embroider-build/content-tag/pull/79) Provide correct types when using cjs with moduleResolution:nodenext ([@ef4](https://github.com/ef4))

#### :house: Internal
* `content-tag`
  * [#69](https://github.com/embroider-build/content-tag/pull/69) Refactor options handling ([@ef4](https://github.com/ef4))
  * [#67](https://github.com/embroider-build/content-tag/pull/67) Fix sourcemap test ([@ef4](https://github.com/ef4))

#### Committers: 1
- Edward Faulkner ([@ef4](https://github.com/ef4))
## Release (2024-02-01)

content-tag 2.0.1 (patch)

#### :bug: Bug Fix
* [#64](https://github.com/embroider-build/content-tag/pull/64) Update type declaration file with latest API ([@vstefanovic97](https://github.com/vstefanovic97))

#### Committers: 1
- Vuk ([@vstefanovic97](https://github.com/vstefanovic97))
## Release (2024-02-01)

content-tag 2.0.0 (major)

#### :boom: Breaking Change
* [#62](https://github.com/embroider-build/content-tag/pull/62) Add js/rust bindings for inline source map generation ([@vstefanovic97](https://github.com/vstefanovic97))

#### :memo: Documentation
* [#63](https://github.com/embroider-build/content-tag/pull/63) Update docs with API changes ([@vstefanovic97](https://github.com/vstefanovic97))
* [#57](https://github.com/embroider-build/content-tag/pull/57) Document API methods (Closes [#45](https://github.com/embroider-build/content-tag/issues/45)) ([@gitKrystan](https://github.com/gitKrystan))

#### :house: Internal
* [#59](https://github.com/embroider-build/content-tag/pull/59) Simplify testing situation to assure import and require works ([@NullVoxPopuli](https://github.com/NullVoxPopuli))

#### Committers: 3
- Krystan HuffMenne ([@gitKrystan](https://github.com/gitKrystan))
- Vuk ([@vstefanovic97](https://github.com/vstefanovic97))
- [@NullVoxPopuli](https://github.com/NullVoxPopuli)
## Release (2023-12-18)

content-tag 1.2.2 (patch)

#### :bug: Bug Fix
* [#54](https://github.com/embroider-build/content-tag/pull/54) fix package.json#exports for types ([@NullVoxPopuli](https://github.com/NullVoxPopuli))

#### Committers: 1
- [@NullVoxPopuli](https://github.com/NullVoxPopuli)
## Release (2023-12-13)

content-tag 1.2.1 (patch)

#### :bug: Bug Fix
* [#52](https://github.com/embroider-build/content-tag/pull/52) Remove extraneous gitignores which interfere with packing ([@NullVoxPopuli](https://github.com/NullVoxPopuli))
* [#50](https://github.com/embroider-build/content-tag/pull/50) Add .npmignore to not ignore pkg/* folders ([@NullVoxPopuli](https://github.com/NullVoxPopuli))

#### :house: Internal
* [#52](https://github.com/embroider-build/content-tag/pull/52) Remove extraneous gitignores which interfere with packing ([@NullVoxPopuli](https://github.com/NullVoxPopuli))
* [#49](https://github.com/embroider-build/content-tag/pull/49) npm pkg fix ([@NullVoxPopuli](https://github.com/NullVoxPopuli))

#### Committers: 1
- [@NullVoxPopuli](https://github.com/NullVoxPopuli)
## Release (2023-12-13)

content-tag 1.2.0 (minor)

#### :rocket: Enhancement
* [#44](https://github.com/embroider-build/content-tag/pull/44) Standalone content-tag implemented via conditional exports in package.json ([@NullVoxPopuli](https://github.com/NullVoxPopuli))

#### :house: Internal
* [#47](https://github.com/embroider-build/content-tag/pull/47) Give the changelog a title ([@NullVoxPopuli](https://github.com/NullVoxPopuli))
* [#46](https://github.com/embroider-build/content-tag/pull/46) Setup release-plan for automated release ([@NullVoxPopuli](https://github.com/NullVoxPopuli))

#### Committers: 1
- [@NullVoxPopuli](https://github.com/NullVoxPopuli)

## 1.1.2 (2023-10-06)

#### :bug: Bug Fix
* [#36](https://github.com/embroider-build/content-tag/pull/36) Updating swc ([@ef4](https://github.com/ef4))

#### Committers: 1
- Edward Faulkner ([@ef4](https://github.com/ef4))

## 1.1.1 (2023-10-02)

#### :bug: Bug Fix
* [#27](https://github.com/embroider-build/content-tag/pull/27) Don't interpret js escapes in hbs ([@ef4](https://github.com/ef4))
* [#24](https://github.com/embroider-build/content-tag/pull/24) BUG: backticks are not escaped ([@NullVoxPopuli](https://github.com/NullVoxPopuli))

#### :house: Internal
* [#33](https://github.com/embroider-build/content-tag/pull/33) Add release-it ([@mansona](https://github.com/mansona))
* [#32](https://github.com/embroider-build/content-tag/pull/32) setup npm credentials for auto-publish ([@mansona](https://github.com/mansona))
* [#28](https://github.com/embroider-build/content-tag/pull/28) Simplified type ([@ef4](https://github.com/ef4))
* [#25](https://github.com/embroider-build/content-tag/pull/25) Move the ContentTagVisitor to its own module ([@ef4](https://github.com/ef4))

#### Committers: 3
- Chris Manson ([@mansona](https://github.com/mansona))
- Edward Faulkner ([@ef4](https://github.com/ef4))
- [@NullVoxPopuli](https://github.com/NullVoxPopuli)


# 1.1.0 (2023-09-22)

 - ENHANCEMENT: added a `parse` method for extracting location information for content tags.

# 1.0.1 (2023-07-21)

 - BUGFIX: inner expressions were not getting transpiled

# 1.0.0 (2023-07-18)

Initial release.
