unity-version-manager-jni
=========================

This project consists of a simple java native interface for the [unity-version-manager].

This repo holds two components:

* a rust JNI module
* a simple library who uses the native interfaces from the rust library

Rust
----

All rust sources are located in the `rust` sub directory. It contains a gradle project which does the rust setup and compiles the JNI interface.

> Note:
> The autosetup of rust does not work on windows system. The plugin can't execute the correct installer for windows.

Java
----

The java library part simply loads the compiled native library based on the platform and provides a static interface to some
of the features provided by [unity-version-manager].


Compile
-------

This is a gradle project so just run:

```bash
./gradlew assemble
```

LICENSE
=======

Copyright 2018 Wooga GmbH

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

<http://www.apache.org/licenses/LICENSE-2.0>

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.


[unity-version-manager]: https:://github.com/Larusso/unity-version-manager
