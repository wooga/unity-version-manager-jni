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
    choice(choices: ["SNAPSHOT", "rc", "final"], description: 'Choose the distribution type', name: 'RELEASE_TYPE')
    choice(choices: ["", "patch", "minor", "major"], description: 'Choose the change scope', name: 'RELEASE_SCOPE')
    choice(choices: ["info", "quiet", "warn", "debug"], description: 'Choose the log level', name: 'LOG_LEVEL')
    booleanParam(defaultValue: false, description: 'Whether to log truncated stacktraces', name: 'STACK_TRACE')
    booleanParam(name: 'SKIP_CHECK', defaultValue: false, description: 'skip verification')
  }

  environment {
    CI = true
    RUST_LOG = "warn,uvm_core=trace,uvm_move_dir=trace,uvm_install2=trace,uvm_jni=trace"
    SNYK_TOKEN = credentials('snyk-wooga-frontend-integration-token')
  }

  stages {
    stage('Preparation') {
      agent any

      steps {
        sendSlackNotification "STARTED", true
      }
    }

    stage("build") {
      failFast false
      parallel {
        stage('osx') {
          agent {
            label "macos && atlas"
          }

          stages {
            stage('assemble') {
              steps {
                gradleWrapper "assemble -Prelease.stage=${params.RELEASE_TYPE.trim()} ${params.RELEASE_SCOPE ? '-Prelease.scope=' + params.RELEASE_SCOPE : ''}"
              }

              post {
                success {
                  archiveArtifacts artifacts: "rust/build/output/*.dylib"
                  stash(name: 'osx_rust', useDefaultExcludes: true, includes: ".gradle/**/*, **/build/**/*", excludes: "build/libs/**")
                }
                failure {
                    archiveArtifacts artifacts: "rust/build/tmp/cargo/compileLibRust/error.txt"
                }
              }
            }

            stage('check') {
              when {
                beforeAgent true
                expression {
                  return params.RELEASE_TYPE == "SNAPSHOT" && !params.SKIP_CHECK
                }
              }

              steps {
                gradleWrapper "check -Prelease.stage=${params.RELEASE_TYPE.trim()} ${params.RELEASE_SCOPE ? '-Prelease.scope=' + params.RELEASE_SCOPE : ''}"
              }

              post {
                success {
                  gradleWrapper "jacocoTestReport"
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

                cleanup {
                    cleanWs()
                }
              }
            }
          }
        }

        stage('linux') {
          agent {
            dockerfile {
              args '-v /home/jenkins_agent/.gradle/init.d:/home/jenkins_agent/.gradle/init.d:ro'
            }
          }

          stages {
            stage('assemble') {
              steps {
                gradleWrapper "assemble -Prelease.stage=${params.RELEASE_TYPE.trim()} ${params.RELEASE_SCOPE ? '-Prelease.scope=' + params.RELEASE_SCOPE : ''}"
              }

              post {
                success {
                  archiveArtifacts artifacts: "rust/build/output/*.so"
                  stash(name: 'linux_lib', useDefaultExcludes: true, includes: "rust/build/output/*.so")
                }
                failure {
                  archiveArtifacts artifacts: "rust/build/tmp/cargo/compileLibRust/error.txt"
                }
              }
            }

            stage('check') {
              when {
                beforeAgent true
                expression {
                  return params.RELEASE_TYPE == "SNAPSHOT" && !params.SKIP_CHECK
                }
              }

              steps {
                gradleWrapper "check -Prelease.stage=${params.RELEASE_TYPE.trim()} ${params.RELEASE_SCOPE ? '-Prelease.scope=' + params.RELEASE_SCOPE : ''}"
              }

              post {
                success {
                  gradleWrapper "jacocoTestReport"
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

                cleanup {
                  cleanWs()
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
                gradleWrapper "assemble -Prelease.stage=${params.RELEASE_TYPE.trim()} ${params.RELEASE_SCOPE ? '-Prelease.scope=' + params.RELEASE_SCOPE : ''}"
              }

              post {
                success {
                  archiveArtifacts artifacts: "rust/build/output/*.dll"
                  stash(name: 'windows_lib', useDefaultExcludes: true, includes: "rust/build/output/*.dll")
                }
                failure {
                    archiveArtifacts artifacts: "rust/build/tmp/cargo/compileLibRust/error.txt"
                }
              }
            }

//            stage('check') {
//              when {
//                beforeAgent true
//                expression {
//                  return (params.RELEASE_TYPE == "SNAPSHOT" && !params.SKIP_CHECK )|| false
//                }
//              }
//
//              steps {
//                echo "skip"
//                //gradleWrapper "check -Prelease.stage=${params.RELEASE_TYPE.trim()} ${params.RELEASE_SCOPE ? '-Prelease.scope=' + params.RELEASE_SCOPE : ''}"
//              }
//
//              post {
//                cleanup {
//                  cleanWs()
//                }
//              }
//            }
          }
        }
      }
    }

    stage('assemble final jar') {
      agent {
        label "macos && atlas"
      }

      steps {
        unstash("osx_rust")
        unstash("windows_lib")
        unstash("linux_lib")

        sh "ls rust/build/output"

        gradleWrapper "assemble -x :rust:assemble -Prelease.stage=${params.RELEASE_TYPE.trim()} ${params.RELEASE_SCOPE ? '-Prelease.scope=' + params.RELEASE_SCOPE : ''}"
      }

      post {
        success {
          archiveArtifacts artifacts: "build/libs/*.jar"
          stash(name: 'final_build', useDefaultExcludes: true, includes: ".gradle/**, **/build/**")
        }
      }
    }

    stage('publish') {
      agent {
        label "macos && atlas"
      }

      environment {
        OSSRH = credentials('ossrh.publish')
        OSSRH_SIGNING_KEY = credentials('ossrh.signing.key')
        OSSRH_SIGNING_KEY_ID = credentials('ossrh.signing.key_id')
        OSSRH_SIGNING_PASSPHRASE = credentials('ossrh.signing.passphrase')
        OSSRH_USERNAME = "${OSSRH_USR}"
        OSSRH_PASSWORD = "${OSSRH_PSW}"
        GRGIT = credentials('github_up')
        GRGIT_USER = "${GRGIT_USR}"
        GRGIT_PASS = "${GRGIT_PSW}"
        GITHUB_LOGIN = "${GRGIT_USR}"
        GITHUB_PASSWORD = "${GRGIT_PSW}"
      }

      steps {
        unstash("final_build")
        gradleWrapper "--info ${params.RELEASE_TYPE.trim()} -Prelease.stage=${params.RELEASE_TYPE.trim()} ${params.RELEASE_SCOPE ? '-Prelease.scope=' + params.RELEASE_SCOPE : ''} -x check -x :rust:copyOut -x :rust:assemble"
      }

      post {
        cleanup {
          cleanWs()
        }
      }
    }
  }

  post {
    always {
      sendSlackNotification currentBuild.result, true
    }
  }
}
