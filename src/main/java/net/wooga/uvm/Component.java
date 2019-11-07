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

public enum Component {
    android(0),
    ios(1),
    tvOs(2),
    webGl(3),
    linux(4),
    windows(5),
    windowsMono(6),
    editor(7),
    mono(8),
    visualStudio(9),
    monoDevelop(10),
    standardAssets(11),
    documentation(12),
    visualStudioProfessionalUnityWorkload(13),
    visualStudioEnterpriseUnityWorkload(14),
    exampleProject(15),
    example(16),
    androidSdkNdkTools(17),
    androidSdkPlatforms(18),
    androidSdkPlatformTools(19),
    androidSdkBuildTools(20),
    androidNdk(21),
    androidOpenJdk(22),
    appleTV(23),
    linuxMono(24),
    mac(25),
    macIL2CPP(26),
    macMono(27),
    metro(28),
    uwpIL2CPP(29),
    uwpNet(30),
    universalWindowsPlatform(31),
    samsungtv(32),
    samsungTV(33),
    tizen(34),
    vuforia(35),
    vuforiaAR(36),
    windowsIL2CCP(37),
    facebook(38),
    facebookGames(39),
    facebookGameRoom(40),
    lumin(41),
    unknown(1000);

    Component(int value) {
        this.value = value;
    }

    private final int value;

    public int value() {
        return value;
    }
}
