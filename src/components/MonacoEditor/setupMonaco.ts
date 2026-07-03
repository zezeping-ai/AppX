import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import cssWorker from "monaco-editor/esm/vs/language/css/css.worker?worker";
import htmlWorker from "monaco-editor/esm/vs/language/html/html.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";

import "monaco-editor/esm/vs/basic-languages/abap/abap.contribution";
import "monaco-editor/esm/vs/basic-languages/apex/apex.contribution";
import "monaco-editor/esm/vs/basic-languages/azcli/azcli.contribution";
import "monaco-editor/esm/vs/basic-languages/bat/bat.contribution";
import "monaco-editor/esm/vs/basic-languages/bicep/bicep.contribution";
import "monaco-editor/esm/vs/basic-languages/clojure/clojure.contribution";
import "monaco-editor/esm/vs/basic-languages/coffee/coffee.contribution";
import "monaco-editor/esm/vs/basic-languages/cpp/cpp.contribution";
import "monaco-editor/esm/vs/basic-languages/csharp/csharp.contribution";
import "monaco-editor/esm/vs/basic-languages/css/css.contribution";
import "monaco-editor/esm/vs/basic-languages/dart/dart.contribution";
import "monaco-editor/esm/vs/basic-languages/dockerfile/dockerfile.contribution";
import "monaco-editor/esm/vs/basic-languages/elixir/elixir.contribution";
import "monaco-editor/esm/vs/basic-languages/flow9/flow9.contribution";
import "monaco-editor/esm/vs/basic-languages/freemarker2/freemarker2.contribution";
import "monaco-editor/esm/vs/basic-languages/fsharp/fsharp.contribution";
import "monaco-editor/esm/vs/basic-languages/go/go.contribution";
import "monaco-editor/esm/vs/basic-languages/graphql/graphql.contribution";
import "monaco-editor/esm/vs/basic-languages/handlebars/handlebars.contribution";
import "monaco-editor/esm/vs/basic-languages/hcl/hcl.contribution";
import "monaco-editor/esm/vs/basic-languages/html/html.contribution";
import "monaco-editor/esm/vs/basic-languages/ini/ini.contribution";
import "monaco-editor/esm/vs/basic-languages/java/java.contribution";
import "monaco-editor/esm/vs/basic-languages/javascript/javascript.contribution";
import "monaco-editor/esm/vs/basic-languages/julia/julia.contribution";
import "monaco-editor/esm/vs/basic-languages/kotlin/kotlin.contribution";
import "monaco-editor/esm/vs/basic-languages/less/less.contribution";
import "monaco-editor/esm/vs/basic-languages/liquid/liquid.contribution";
import "monaco-editor/esm/vs/basic-languages/lua/lua.contribution";
import "monaco-editor/esm/vs/basic-languages/markdown/markdown.contribution";
import "monaco-editor/esm/vs/basic-languages/msdax/msdax.contribution";
import "monaco-editor/esm/vs/basic-languages/mysql/mysql.contribution";
import "monaco-editor/esm/vs/basic-languages/objective-c/objective-c.contribution";
import "monaco-editor/esm/vs/basic-languages/pascal/pascal.contribution";
import "monaco-editor/esm/vs/basic-languages/perl/perl.contribution";
import "monaco-editor/esm/vs/basic-languages/pgsql/pgsql.contribution";
import "monaco-editor/esm/vs/basic-languages/php/php.contribution";
import "monaco-editor/esm/vs/basic-languages/powershell/powershell.contribution";
import "monaco-editor/esm/vs/basic-languages/protobuf/protobuf.contribution";
import "monaco-editor/esm/vs/basic-languages/python/python.contribution";
import "monaco-editor/esm/vs/basic-languages/r/r.contribution";
import "monaco-editor/esm/vs/basic-languages/redis/redis.contribution";
import "monaco-editor/esm/vs/basic-languages/ruby/ruby.contribution";
import "monaco-editor/esm/vs/basic-languages/rust/rust.contribution";
import "monaco-editor/esm/vs/basic-languages/scala/scala.contribution";
import "monaco-editor/esm/vs/basic-languages/shell/shell.contribution";
import "monaco-editor/esm/vs/basic-languages/sql/sql.contribution";
import "monaco-editor/esm/vs/basic-languages/swift/swift.contribution";
import "monaco-editor/esm/vs/basic-languages/typescript/typescript.contribution";
import "monaco-editor/esm/vs/basic-languages/xml/xml.contribution";
import "monaco-editor/esm/vs/basic-languages/yaml/yaml.contribution";

let configured = false;

export function ensureMonacoEnvironment() {
  if (configured || typeof globalThis === "undefined") {
    return;
  }

  globalThis.MonacoEnvironment = {
    getWorker(_: unknown, label: string) {
      if (label === "json") {
        return new jsonWorker();
      }
      if (label === "css" || label === "scss" || label === "less") {
        return new cssWorker();
      }
      if (label === "html" || label === "handlebars" || label === "razor") {
        return new htmlWorker();
      }
      if (label === "typescript" || label === "javascript") {
        return new tsWorker();
      }
      return new editorWorker();
    },
  };

  configured = true;
}
