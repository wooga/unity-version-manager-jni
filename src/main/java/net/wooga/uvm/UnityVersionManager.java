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

package net.wooga.uvm;

import java.io.File;

/**
 * This is a simple native interface for the {@code unity version manager} tool.
 * It loads the needed dynamic library from the resource directory at startup time.
 */
public class UnityVersionManager {

    static {
        NativeLoader.loadLibrary(UnityVersionManager.class.getClassLoader(), System.mapLibraryName("uvm_jni"));
    }

    /**
     * Returns the uvm core library version.
     *
     * @return the uvm core library version
     */
    public static native String uvmVersion();

    /**
     * Lists Unity installations.
     *
     * @return an array of {@code Installation} or null
     */
    public static native Installation[] listInstallations();

    /**
     * Detects the unity editor version used in {@code projectPath}.
     *
     * @param projectPath the path to the unity project root
     * @return a version string or {@code NULL}
     */
    public static native String detectProjectVersion(File projectPath);

    /**
     * Returns the path to the installation location for the provided version or {@code Null}.
     *
     * @param unityVersion the version string to fetch the installation location for
     * @return a {@code Installation} object or null
     */
    public static native Installation locateUnityInstallation(String unityVersion);

    /**
     * Installs the given version of unity to destination.
     * <p>
     * If the unity version is already installed, returns early.
     *
     * @param version     the version of unity to install
     * @param destination the location to install unity to
     * @return a {@code Installation} object or null
     */
    public static native Installation installUnityEditor(String version, File destination);

    /**
     * Installs the given version of unity to default destination.
     * <p>
     * If the unity version is already installed, returns early.
     *
     * @param version the version of unity to install
     * @return a {@code Installation} object or null
     */
    public static native Installation installUnityEditor(String version);

    /**
     * Installs the given version of unity and additional components to default destination.
     * <p>
     * If the unity version and all requested components are already installed, returns early.
     *
     * @param version    the version of unity to install
     * @param components a list of optional {@code Component}s to install
     * @return a {@code Installation} object or null
     * @see Component
     * @see Installation
     */
    public static native Installation installUnityEditor(String version, Component[] components);

    /**
     * Installs the given version of unity and additional components to destination.
     * <p>
     * If the unity version and all requested components are already installed, returns early.
     *
     * @param version     the version of unity to install
     * @param destination the location to install unity to
     * @param components  a list of optional {@code Component}s to install
     * @return a {@code Installation} object or null
     * @see Component
     * @see Installation
     */
    public static native Installation installUnityEditor(String version, File destination, Component[] components);


    /**
     * Return the version as {@code String} of the unity installation at the provided location or {@code Null}.
     *
     * @param installationLocation the path to the unity installation or Unity executable
     * @return a version string or {@code null}
     */
    public static String readUnityVersion(File installationLocation) {
        Installation installation = Installation.atLocation(installationLocation);
        if (installation != null) {
            return installation.getVersion();
        }
        return null;
    }
}
