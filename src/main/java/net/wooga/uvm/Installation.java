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

public class Installation {
    static {
        NativeLoader.loadLibrary(Installation.class.getClassLoader(), System.mapLibraryName("uvm_jni"));
    }

    private File location;
    private String version;

    public Installation(File location, String version) {
        this.location = location;
        this.version = version;
    }

    public File getLocation() {
        return location;
    }

    public String getVersion() {
        return version;
    }

    @Override
    public boolean equals(Object obj) {
        if (obj instanceof Installation) {
            Installation other = (Installation) obj;
            return other.location.equals(this.location)
                    && other.version.equals(this.version);
        }
        return false;
    }

    public native Component[] getComponents();

    /**
     * Returns the path to the unity executable.
     * <p>
     * The path to the unity executable is system depended.
     * <p>
     * * on macOS: {@code Unity.app/Contents/MacOS/Unity}
     * <p>
     * * on windows: {@code Editor\\Unity.exe}
     *
     * @return the path to the executable
     */
    public native File getExecutable();

    /**
     * Returns a @{code Installation} object from given path.
     *
     * @param installationLocation the path to the unity installation
     * @return a @{code Installation} object or @{code null} if installation can't be found
     */
    public static native Installation atLocation(File installationLocation);
}
