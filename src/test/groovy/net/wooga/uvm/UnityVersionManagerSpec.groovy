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

import com.wooga.spock.extensions.uvm.UnityInstallation
import spock.lang.Ignore
import spock.lang.IgnoreIf
import spock.lang.Shared
import spock.lang.Specification
import spock.lang.Unroll

import java.nio.file.Files
import java.util.concurrent.ForkJoinPool

class UnityVersionManagerSpec extends Specification {

    @Shared
    File buildDir

    def setup() {
        buildDir = new File('build/unityVersionManagerSpec')
        buildDir.mkdirs()
    }

    @Unroll("can call :#method on UnityVersionManager")
    def "unity version manager interface doesn't crash"() {
        when:
        (UnityVersionManager.class).invokeMethod(method, *arguments)
        then:
        noExceptionThrown()

        where:
        method                    | arguments
        "uvmVersion"              | null
        "listInstallations"       | null
        "detectProjectVersion"    | [new File("")]
        "locateUnityInstallation" | null
    }

    def "can call :installUnityEditor on UnityVersionManager"() {
        when:
        UnityVersionManager.installUnityEditor("")
        then:
        noExceptionThrown()
        when:
        UnityVersionManager.installUnityEditor("", new File(""))
        then:
        noExceptionThrown()
        when:
        UnityVersionManager.installUnityEditor("", [] as Component[])
        then:
        noExceptionThrown()
        when:
        UnityVersionManager.installUnityEditor(null, null, null)
        then:
        noExceptionThrown()
    }

    File mockUnityProject(String editorVersion) {
        def outerDir = File.createTempDir("uvm_jni_projects_", "_base_path")
        def projectDir = new File(outerDir, "unity_testproject")
        def projectSettings = new File(projectDir, "ProjectSettings")
        projectSettings.mkdirs()

        def projectVersion = new File(projectSettings, "ProjectVersion.txt")
        projectVersion << "m_EditorVersion: ${editorVersion}"
        projectDir
    }

    @Unroll
    def "detectProjectVersion returns #resultMessage when #reason"() {
        expect:
        UnityVersionManager.detectProjectVersion(path) == expectedResult

        where:
        path                                      | reason                                                  | expectedResult
        null                                      | "path is null"                                          | null
        File.createTempDir()                      | "path points to an invalid location"                    | null
        mockUnityProject("2018.2b4")              | "editor version is invalid"                             | null
        mockUnityProject("2018.2.1b4")            | "path points to a unity project location"               | "2018.2.1b4"
        mockUnityProject("2017.1.2f3").parentFile | "path points to a directory containing a unity project" | "2017.1.2f3"
        resultMessage = expectedResult ? "the editor version" : "null"
    }

    static String OS = System.getProperty("os.name").toLowerCase()
    static boolean isWindows() {
        return (OS.indexOf("win") >= 0)
    }

    static boolean isMac() {
        return (OS.indexOf("mac") >= 0)
    }

    static boolean isLinux() {
        return (OS.indexOf("linux") >= 0)
    }

    @Shared
    @UnityInstallation(version="2018.4.19f1", basePath = "build/unity", cleanup = true)
    Installation preInstalledUnity2018_4_19f1

    @Unroll
    def "locateUnityInstallation returns #resultMessage when #reason"() {
        expect:
        UnityVersionManager.locateUnityInstallation(version) == expectedResult

        where:
        version                          | reason                          | expectedResult
        null                             | "version is null"               | null
        preInstalledUnity2018_4_19f1.version                    | "when version is installed"     | preInstalledUnity2018_4_19f1
        "1.1.1f1"                        | "when version is not installed" | null
        "2018.0.1"                       | "when version is invalid"       | null

        resultMessage = expectedResult ? "the unity location" : "null"
    }

    @Unroll
    def "listInstallations returns list of installed versions"() {
        given: "some installed versions"
        def v = preInstalledUnity2018_4_19f1.version

        when: "fetch installations"
        def installations = UnityVersionManager.listInstallations()

        then:
        installations != null
        def versions = installations.collect { it.version }
        v.each { version ->
            versions.contains(version)
        }
    }

    def "installUnityEditor installs unity to location"() {
        given: "a version to install"
        def version = "2019.3.0a5"
        assert !UnityVersionManager.listInstallations().collect({ it.version }).contains(version)

        and: "a temp install location"
        def basedir = Files.createTempDirectory(buildDir.toPath(), "installUnityEditor_without_components").toFile()
        def destination = new File(basedir, version)
        assert !destination.exists()

        when:
        def result = UnityVersionManager.installUnityEditor(version, destination)

        then:
        result != null
        result.location.exists()
        result.location.absolutePath == destination.absolutePath
        result.version == version
        def installation = UnityVersionManager.locateUnityInstallation(version)
        installation.location.absolutePath == result.location.absolutePath

        cleanup:
        destination.deleteDir()
    }

    @IgnoreIf({env.containsKey("CI")})
    def "installUnityEditor installs unity to default location"() {
        given: "a version to install"
        def version = "2019.3.0a5"
        assert !UnityVersionManager.listInstallations().collect({ it.version }).contains(version)

        when:
        def result = UnityVersionManager.installUnityEditor(version)

        then:
        result != null
        result.location.exists()
        result.version == version
        def installation = UnityVersionManager.locateUnityInstallation(version)
        installation.location.absolutePath == result.location.absolutePath

        cleanup:
        result.location.deleteDir()
    }

    def "installUnityEditor installs unity and components to location"() {
        given: "a version to install"
        def version = "2019.3.0a5"
        assert !UnityVersionManager.listInstallations().collect({ it.version }).contains(version)

        and: "a temp install location"
        def basedir = Files.createTempDirectory(buildDir.toPath(), "installUnityEditor_with_components").toFile()
        def destination = new File(basedir, version)
        assert !destination.exists()

        and: "no engines"
        def playbackEnginesPath = (isWindows() || isLinux()) ? "Editor/Data/PlaybackEngines" : "PlaybackEngines"
        def playbackEngines = new File(destination, playbackEnginesPath)
        assert !playbackEngines.exists()

        when:
        def result = UnityVersionManager.installUnityEditor(version, destination, [Component.android, Component.ios].toArray() as Component[])

        then:
        result != null
        result.location.exists()
        result.location.absolutePath == destination.absolutePath
        result.version == version

        destination.exists()
        playbackEngines.exists()
        new File(playbackEngines, "iOSSupport").exists()
        new File(playbackEngines, "AndroidPlayer").exists()
        def installation = UnityVersionManager.locateUnityInstallation(version)
        installation.location.absolutePath == result.location.absolutePath

        cleanup:
        destination.deleteDir()
    }

    def "locks process when a different process is installing the same version"() {
        given: "a version to install"
        def version = "2019.3.0a5"
        assert !UnityVersionManager.listInstallations().collect({ it.version }).contains(version)

        and:
        def threads = ["2019.3.0a5", "2019.3.0a5"].collect { v ->
            Thread.start({
                UnityVersionManager.installUnityEditor(v)
            })
        }

        when:
        threads*.join()

        then:
        noExceptionThrown()
        UnityVersionManager.locateUnityInstallation(version) != null

        cleanup:
        UnityVersionManager.locateUnityInstallation(version).location.deleteDir()

    }

    @IgnoreIf({env.containsKey("CI")})
    def "installUnityEditor installs unity and components to default location"() {


        given: "a version to install"
        def version = "2019.3.0a5"
        assert !UnityVersionManager.listInstallations().collect({ it.version }).contains(version)

        when:
        def result = UnityVersionManager.installUnityEditor(version, [Component.android, Component.ios].toArray() as Component[])

        then:
        result != null
        result.location.exists()
        result.version == version
        UnityVersionManager.listInstallations().collect({ it.version }).contains(version)
        def playbackEngines = new File(result.location, "PlaybackEngines")
        playbackEngines.exists()
        new File(playbackEngines, "iOSSupport").exists()
        new File(playbackEngines, "AndroidPlayer").exists()
        def installation = UnityVersionManager.locateUnityInstallation(version)
        installation.location.absolutePath == result.location.absolutePath

        cleanup:
        result.location.deleteDir()
    }

    def "returns unity version at location"() {
        given: "a installation"
        def installation = preInstalledUnity2018_4_19f1
        assert installation

        when:
        def result = UnityVersionManager.readUnityVersion(installation.location)

        then:
        result == installation.version
    }
}
