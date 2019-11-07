/*
 * Copyright 2018 Wooga GmbH
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *       http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package net.wooga.uvm

import spock.lang.Issue
import spock.lang.Shared
import spock.lang.Specification
import spock.lang.Unroll

import java.nio.file.Files

class InstallationSpec extends Specification {

    @Shared
    File buildDir

    def setup() {
        buildDir = new File('build/unityVersionManagerSpec')
        buildDir.mkdirs()
    }


    @Unroll("can call :#method on Installation")
    def "installation interface doesn't crash"() {
        given: "An Installation object"
        def installation = new Installation(null, null)

        when:
        installation.invokeMethod(method, *arguments)
        then:
        noExceptionThrown()

        where:
        method          | arguments
        "getComponents" | null
    }

    @Unroll("method :getComponents returns #valueMessage #reason")
    def "get components returns list of installed components"() {
        given: "install unity with specific components"
        def basedir = Files.createTempDirectory(buildDir.toPath(), "installationSpec_getComponents").toFile()
        basedir.deleteOnExit()
        def destination = new File(basedir, version)
        assert !destination.exists()
        def installation = UnityVersionManager.installUnityEditor(version, destination, components.toArray() as Component[])
        assert destination.exists()

        expect:
        installation
        installation.components.toList().containsAll(expectedComponents)

        cleanup:
        destination.deleteDir()

        where:
        components                         | expectedComponents                                                                                                                                                                                                 | reason
        [Component.android, Component.ios] | [Component.android, Component.androidNdk, Component.androidOpenJdk, Component.androidSdkBuildTools, Component.androidSdkNdkTools, Component.androidSdkPlatforms, Component.androidSdkPlatformTools, Component.ios] | "when components are installed"
        []                                 | []                                                                                                                                                                                                                 | "when no components are installed"
        version = "2019.3.0b8"
        valueMessage = components.size() > 0 ? "list of installed components" : "empty list"
    }

    @Unroll("can fetch installation at location")
    def "get a installation from a File location"() {
        given:
        def basedir = Files.createTempDirectory(buildDir.toPath(), "installationSpec_installation_at_location").toFile()
        basedir.deleteOnExit()
        def destination = new File(basedir, version)
        assert !destination.exists()
        def installation = UnityVersionManager.installUnityEditor(version, destination)
        assert destination.exists()

        expect:
        def installation2 = Installation.atLocation(destination)
        installation2 != null
        installation2 == installation
        !installation2.is(installation)

        cleanup:
        destination.deleteDir()

        where:
        version = "2019.3.0a5"
    }

    @Issue("https://github.com/wooga/unity-version-manager-jni/issues/21")
    @Unroll("can fetch installation with executable path #message")
    def "get a installation from a exec path"() {
        given:
        def basedir = Files.createTempDirectory(buildDir.toPath(), "installationSpec_installation_at_location").toFile()
        basedir.deleteOnExit()
        def destination = versionInPath ? new File(basedir, version) : basedir
        assert !destination.exists()
        def installation = UnityVersionManager.installUnityEditor(version, destination)
        assert destination.exists()

        expect:
        def installation2 = Installation.atLocation(installation.executable)

        installation2 != null
        installation2 == installation
        !installation2.is(installation)

        cleanup:
        destination.deleteDir()

        where:
        version = "2019.3.0a5"
        versionInPath = [true, false]
        message = versionInPath ? "" : "without version information in destination path"
    }

    def "return null when installation at location doesn't exist"() {
        expect:
        !Installation.atLocation(File.createTempDir())
    }

    File expectedUnityExecutable(File installationLocation) {
        String os = System.getProperty("os.name").toLowerCase()
        if (os.indexOf("win") >= 0) {
            return new File(installationLocation, "Editor\\Unity.exe")
        } else if (os.indexOf("mac") >= 0) {
            return new File(installationLocation, "Unity.app/Contents/MacOS/Unity")
        } else if (os.indexOf("linux") >= 0) {
            return new File(installationLocation, "Editor/Unity")
        }
        return null
    }

    @Unroll("returns path to unity executable")
    def "returns path to executable"() {
        given:
        def basedir = Files.createTempDirectory(buildDir.toPath(), "installationSpec_path_to_executable").toFile()
        basedir.deleteOnExit()
        def destination = new File(basedir, version)
        assert !destination.exists()
        def installation = UnityVersionManager.installUnityEditor(version, destination)
        assert destination.exists()

        expect:
        installation.executable.absoluteFile == expectedUnityExecutable(destination).absoluteFile

        cleanup:
        destination.deleteDir()

        where:
        version = "2019.3.0a5"
    }
}
