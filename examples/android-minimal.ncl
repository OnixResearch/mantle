# Java-source APK proof input. The rail materializes these exact three source
# members and supplies the separately generated keystore/password as store inputs.
let android = import "../lib/android.ncl" in
{
  plan = android.mkApk {
    module = "hello-world",
    application_id = "org.example.mantlehello",
    version_code = 1,
    manifest_member = "AndroidManifest.xml",
    resources = ["res/values/strings.xml"],
    java_sources = ["src/org/example/mantlehello/MainActivity.java"],
    signing = {
      keystore_member = "keys/release.jks",
      key_alias = "mantle",
      schemes = ["v2"],
    },
  },
  manifest = "<manifest xmlns:android=\"http://schemas.android.com/apk/res/android\" package=\"org.example.mantlehello\"><uses-sdk android:minSdkVersion=\"24\" android:targetSdkVersion=\"35\"/><application android:label=\"@string/app_name\"><activity android:name=\".MainActivity\" android:exported=\"true\"><intent-filter><action android:name=\"android.intent.action.MAIN\"/><category android:name=\"android.intent.category.LAUNCHER\"/></intent-filter></activity></application></manifest>\n",
  java = "package org.example.mantlehello;\nimport android.app.Activity;\nimport android.os.Bundle;\nimport android.widget.TextView;\npublic final class MainActivity extends Activity {\n    @Override public void onCreate(Bundle state) {\n        super.onCreate(state);\n        TextView message = new TextView(this);\n        message.setText(\"Hello, Mantle!\");\n        setContentView(message);\n    }\n}\n",
  resources_xml = "<resources><string name=\"app_name\">Mantle HelloWorld</string></resources>\n",
}
