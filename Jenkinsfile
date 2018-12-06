#!groovy
@Library('github.com/wooga/atlas-jenkins-pipeline@1.x') _

/*
 * Copyright 2018 Wooga
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

pipeline {
  agent none

  options {
    buildDiscarder(logRotator(artifactNumToKeepStr:'40'))
  }

  parameters {
    choice(choices: "SNAPSHOT\nrc\nfinal", description: 'Choose the distribution type', name: 'RELEASE_TYPE')
    choice(choices: "patch\nminor\nmajor", description: 'Choose the change scope', name: 'RELEASE_SCOPE')
  }

  stages {
    stage('Preparation') {
      agent any

      steps {
        sendSlackNotification "STARTED", true
      }
    }

    stage("build") {
      failFast true
      parallel {
        stage('osx') {
          agent {
            label "osx && atlas"
          }

          stages {
            stage('assemble') {
              steps {
                gradleWrapper "assemble"
              }

              post {
                success {
                  archiveArtifacts artifacts: "rust/build/output/*.dylib"
                  stash(name: 'osx_rust', useDefaultExcludes: true, includes: ".gradle/**/*, **/build/**/*", excludes: "build/libs/**")
                }
              }
            }

            stage('check') {
              when {
                beforeAgent true
                expression {
                  return params.RELEASE_TYPE == "SNAPSHOT"
                }
              }

              steps {
                gradleWrapper "check"
              }

              post {
                success {
                  gradleWrapper "jacocoTestReport coveralls"
                  publishHTML([
                    allowMissing: true,
                    alwaysLinkToLastBuild: true,
                    keepAll: true,
                    reportDir: 'build/reports/jacoco/test/html',
                    reportFiles: 'index.html',
                    reportName: 'Coverage',
                    reportTitles: ''
                    ])
                }

                always {
                  junit allowEmptyResults: true, testResults: '**/build/test-results/**/*.xml'
                }
              }
            }
          }
        }

        stage('windows') {
          agent {
            label "windows && atlas"
          }

          stages {
            stage('assemble') {
              steps {
                gradleWrapper "assemble"
              }

              post {
                success {
                  archiveArtifacts artifacts: "rust/build/output/*.dll"
                  stash(name: 'windows_lib', useDefaultExcludes: true, includes: "rust/build/output/*.dll")
                }
              }
            }

            stage('check') {
              when {
                beforeAgent true
                expression {
                  return params.RELEASE_TYPE == "SNAPSHOT"
                }
              }

              steps {
                gradleWrapper "check"
              }

              post {
                success {
                  gradleWrapper "jacocoTestReport coveralls"
                  publishHTML([
                    allowMissing: true,
                    alwaysLinkToLastBuild: true,
                    keepAll: true,
                    reportDir: 'build/reports/jacoco/test/html',
                    reportFiles: 'index.html',
                    reportName: 'Coverage',
                    reportTitles: ''
                    ])
                }

                always {
                  junit allowEmptyResults: true, testResults: '**/build/test-results/**/*.xml'
                }
              }
            }
          }
        }
      }
    }

    stage('assemble final jar') {
      agent {
        label "osx && atlas"
      }

      steps {
        unstash("osx_rust")
        unstash("windows_lib")

        gradleWrapper "assemble -Prelease.stage=${params.RELEASE_TYPE.trim()} -Prelease.scope=${params.RELEASE_SCOPE}"
      }

      post {
        success {
          archiveArtifacts artifacts: "build/libs/*.jar"
          stash(name: 'final_build', useDefaultExcludes: true, includes: ".gradle/**, **/build/**")
        }
      }
    }

    stage('publish') {
      when {
        beforeAgent true
        expression {
          return params.RELEASE_TYPE != "SNAPSHOT"
        }
      }

      agent {
        label "osx && atlas"
      }

      environment {
        BINTRAY               = credentials('bintray.publish')
        GRGIT                 = credentials('github_up')

        BINTRAY_USER          = "${BINTRAY_USR}"
        BINTRAY_API_KEY       = "${BINTRAY_PSW}"
        GRGIT_USER            = "${GRGIT_USR}"
        GRGIT_PASS            = "${GRGIT_PSW}"
        GITHUB_LOGIN          = "${GRGIT_USR}"
        GITHUB_PASSWORD       = "${GRGIT_PSW}"
      }

      steps {
        unstash("final_build")
        gradleWrapper "${params.RELEASE_TYPE.trim().toLowerCase()} -Pbintray.user=${BINTRAY_USER} -Pbintray.key=${BINTRAY_API_KEY} -Prelease.stage=${params.RELEASE_TYPE.trim()} -Prelease.scope=${params.RELEASE_SCOPE} -x check"
      }
    }
  }

  post {
    always {
      sendSlackNotification currentBuild.result, true
    }
  }
}
