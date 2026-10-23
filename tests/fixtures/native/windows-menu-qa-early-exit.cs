// tests/fixtures/native/windows-menu-qa-early-exit.cs

namespace XdzipOwnedMenuQa {
    internal static class HelperEarlyExitFixture {
        private static string stage="not_started";
        private static MenuQaOptions activeValidationOptions;
        private sealed class FixtureAssertionException : System.InvalidOperationException {
            internal readonly string Code;
            internal FixtureAssertionException(string code) : base(code) { Code=code; }
        }

        private static void Require(bool condition, string reason) {
            if (!condition) throw new FixtureAssertionException(reason);
        }

        private static System.IO.StreamReader ReleaseReader(System.IO.Stream stream) {
            // Process.StandardInput may emit its UTF-8 preamble at construction.
            // Decode that preamble explicitly; the RELEASE token remains exact.
            return new System.IO.StreamReader(stream,new System.Text.UTF8Encoding(false,true),true);
        }

        private static void Core(MenuQaOptions options, System.Collections.Generic.Dictionary<string, object> result) {
            stage="core_identity_guards";
            int rejected = 0;
            var exited = new MenuQaLauncher.ChildIdentityObservation { Exited=true, ExitCode=7 };
            Require(!MenuQaLauncher.ValidateObservedChildIdentity(exited, options.Tests,
                delegate(string reason, int? code, string type) {
                    Require(reason == "child_exited_before_identity" && code == 7, "Early-exit identity evidence lost.");
                    ++rejected;
                }) && rejected == 1, "Exited child was accepted.");
            rejected = 0;
            Require(!MenuQaLauncher.ValidateObservedChildIdentity(new MenuQaLauncher.ChildIdentityObservation(), options.Tests,
                delegate(string reason, int? code, string type) {
                    Require(reason == "child_identity_unavailable", "Empty-path reason differs."); ++rejected;
                }) && rejected == 1, "Empty image identity was accepted.");
            string oldException = null;
            try { System.IO.Path.GetFullPath(""); }
            catch (System.Exception error) { oldException = error.GetType().FullName; }
            Require(oldException == "System.ArgumentException", "Old empty-path branch did not reproduce.");
            result["static_identity_guards_passed"]=true;
            result["old_empty_path_exception_type"]=oldException;
            stage="core_deferred_exception_guard";
            int deferredErrors = 0;
            MenuQaLauncher.RunDeferred(delegate { throw new System.ArgumentException("Owned diagnostic exception."); },
                delegate(System.Exception error) {
                    Require(error is System.ArgumentException, "Deferred exception type lost."); ++deferredErrors;
                });
            Require(deferredErrors == 1, "Deferred callback escaped or failed twice.");
            result["safe_deferred_exception_routing_passed"]=true;
            stage="core_frame_contract_guards";
            var valid = new System.Collections.Generic.Dictionary<string, object> {
                {options.Contract.ReferenceGeneration,1}, {"shadow_same_frame",true},
                {"capture_stage",options.Contract.CaptureStage}, {"graphics_api",options.Contract.GraphicsApi},
                {"configuration",options.Contract.Configuration}, {"platform",options.Contract.Platform},
                {"revision",options.Source.Substring(0,12)}, {"trigger_to_complete_frame_ms",20}
            };
            foreach (string field in options.Contract.GenerationFields) valid[field]=1;
            Require(options.Contract.Accepts(valid, options.Source), "Valid frame contract rejected.");
            valid[options.Contract.ReferenceGeneration]=1.5;
            foreach (string field in options.Contract.GenerationFields) valid[field]=1.5;
            Require(!options.Contract.Accepts(valid, options.Source), "Fractional frame generation accepted.");
            valid[options.Contract.ReferenceGeneration]=1;
            foreach (string field in options.Contract.GenerationFields) valid[field]=1;
            valid[options.Contract.GenerationFields[0]]=2;
            Require(!options.Contract.Accepts(valid, options.Source), "Later frame accepted.");
            valid[options.Contract.GenerationFields[0]]=1; valid["trigger_to_complete_frame_ms"]=System.Double.NaN;
            Require(!options.Contract.Accepts(valid, options.Source), "Non-finite timing accepted.");
            valid["trigger_to_complete_frame_ms"]=20; valid["shadow_same_frame"]=false;
            Require(!options.Contract.Accepts(valid, options.Source), "Missing same-frame shadow accepted.");
            result["frame_generation_shadow_and_finite_timing_guards_passed"]=true;
            stage="core_build_path_guard";
            bool outsideRejected=false;
            try { MenuQaOptions.RequireBuildChild(options.Root, System.IO.Path.Combine(options.Root,"build-sibling","escape")); }
            catch (System.ArgumentException) { outsideRejected=true; }
            Require(outsideRejected, "Build prefix sibling accepted.");
            result["build_path_boundary_guard_passed"]=true;
            stage="core_utf8_bom_release_guard";
            byte[] protocol=System.Text.Encoding.UTF8.GetBytes("\uFEFFRELEASE\n");
            string legacyToken, decodedToken;
            using (var legacy=new System.IO.StreamReader(new System.IO.MemoryStream(protocol),
                new System.Text.UTF8Encoding(false,true),false)) legacyToken=legacy.ReadLine();
            using (var reader=ReleaseReader(new System.IO.MemoryStream(protocol))) decodedToken=reader.ReadLine();
            Require(legacyToken=="\uFEFFRELEASE" && legacyToken!="RELEASE", "Legacy BOM token branch did not reproduce.");
            Require(decodedToken=="RELEASE", "BOM-aware fixed release token failed.");
            result["legacy_utf8_bom_fixed_token_rejected"]=true;
            result["shared_bom_aware_release_reader_passed"]=true;
        }

        private static void Full(MenuQaOptions options, System.Collections.Generic.Dictionary<string, object> result) {
            stage="full_source_binding";
            activeValidationOptions=options;
            options.ValidateFull();
            result["full_source_binding_passed"]=true;
            string fixtureBinary=System.Reflection.Assembly.GetExecutingAssembly().Location;
            var liveInfo=new System.Diagnostics.ProcessStartInfo(fixtureBinary,"--owned-live-identity") {
                UseShellExecute=false, CreateNoWindow=true, RedirectStandardInput=true, RedirectStandardOutput=true
            };
            using (var live=System.Diagnostics.Process.Start(liveInfo)) {
                stage="live_owned_wait_ready";
                System.IntPtr liveHandle=live.Handle;
                var ready=live.StandardOutput.ReadLineAsync();
                Require(ready.Wait(5000) && ready.Result == "READY", "Live fixture readiness handshake timed out.");
                stage="live_owned_query_identity";
                var observation=MenuQaLauncher.ObserveOwnedChildIdentity(live);
                int rejects=0;
                bool accepted=MenuQaLauncher.ValidateObservedChildIdentity(observation,fixtureBinary,
                    delegate(string reason,int? code,string type) { ++rejects; });
                stage="live_owned_release";
                result["live_owned_stdin_encoding"]=live.StandardInput.Encoding.WebName;
                result["live_owned_stdin_preamble_hex"]=System.BitConverter.ToString(live.StandardInput.Encoding.GetPreamble()).Replace("-","");
                live.StandardInput.WriteLine("RELEASE"); live.StandardInput.Flush(); live.StandardInput.Close();
                bool released=live.WaitForExit(5000);
                result["live_owned_release_child_exited"]=released;
                result["live_owned_release_child_exit_code"]=released?(object)live.ExitCode:null;
                Require(released && live.ExitCode==0, "Live fixture release handshake failed.");
                Require(accepted && rejects==0 && !observation.Exited && observation.Win32Error==0,
                    "Live owned handle image query rejected.");
                result["live_owned_handle_query_api"]="QueryFullProcessImageNameW";
                result["live_owned_readiness_and_release_handshake_passed"]=true;
                result["live_owned_release_reader"]="UTF8 with BOM detection";
            }
            var fastInfo=new System.Diagnostics.ProcessStartInfo(fixtureBinary,"--owned-fast-exit") {
                UseShellExecute=false, CreateNoWindow=true
            };
            using (var owned=System.Diagnostics.Process.Start(fastInfo)) {
                stage="fast_owned_wait_exit";
                System.IntPtr ownedHandle=owned.Handle;
                Require(owned.WaitForExit(5000) && owned.ExitCode==7, "Fast owned fixture did not exit as expected.");
                stage="fast_owned_legacy_branch";
                string originalException=null;
                try {
                    string actualPath=null;
                    for (int attempt=0; attempt<20 && !owned.HasExited; ++attempt) {
                        try { actualPath=owned.MainModule.FileName; break; }
                        catch (System.ComponentModel.Win32Exception) { System.Threading.Thread.Sleep(5); }
                    }
                    System.IO.Path.GetFullPath(actualPath ?? "");
                } catch (System.Exception error) { originalException=error.GetType().FullName; }
                Require(originalException=="System.ArgumentException", "Exact exited-child legacy branch did not reproduce.");
                var observation=MenuQaLauncher.ObserveOwnedChildIdentity(owned);
                Require(observation.Exited && observation.ExitCode==7 && observation.ImagePath==null,
                    "Early-exit observation differs.");
                result["real_early_exit_identity_observed"]=true;
                stage="controlled_form_construction";
                MenuQaOptions formOptions=options.ForOutput(System.IO.Path.Combine(options.Output,"controlled-fail"));
                activeValidationOptions=formOptions;
                using (var form=new MenuQaLauncher(formOptions)) {
                    stage="controlled_form_identity_rejection";
                    var flags=System.Reflection.BindingFlags.Instance|System.Reflection.BindingFlags.NonPublic;
                    System.Action<string,object> set=delegate(string name,object value) {
                        typeof(MenuQaLauncher).GetField(name,flags).SetValue(form,value);
                    };
                    set("child",owned); set("childHandle",ownedHandle); set("childPid",(uint)owned.Id);
                    set("currentCase","diagnostic-owned-fast-exit"); set("currentStage","validating_owned_child_identity");
                    var failure=typeof(MenuQaLauncher).GetMethod("Fail",flags);
                    int callbacks=0;
                    bool accepted=MenuQaLauncher.ValidateObservedChildIdentity(observation,options.Tests,
                        delegate(string reason,int? code,string type) {
                            Require(code==7, "Identity rejection lost exit code."); ++callbacks;
                            failure.Invoke(form,new object[]{reason,"诊断夹具，受控停止。",type});
                        });
                    Require(!accepted && callbacks==1 && !form.Visible, "Controlled failure was not invisible and singular.");
                    var serializer=new System.Web.Script.Serialization.JavaScriptSerializer();
                    var report=serializer.Deserialize<System.Collections.Generic.Dictionary<string,object>>(
                        System.IO.File.ReadAllText(System.IO.Path.Combine(formOptions.Output,"review.json")));
                    var failures=(System.Collections.ArrayList)report["failures"];
                    stage="controlled_form_failure_record";
                    var first=(System.Collections.Generic.Dictionary<string,object>)failures[0];
                    Require(System.Convert.ToString(report["status"])=="failed" &&
                        System.Convert.ToString(first["reason"])=="child_exited_before_identity" &&
                        System.Convert.ToInt32(first["child_exit_code"])==7 &&
                        !System.Convert.ToBoolean(first["termination_attempted"]), "Controlled fail lost normal process exit.");
                    result["controlled_form_fail_reason"]=first["reason"];
                    result["controlled_form_fail_exit_code"]=first["child_exit_code"];
                    result["termination_attempted"]=first["termination_attempted"];
                    result["visible_form_shown"]=form.Visible;
                }
                result["exact_real_exited_child_old_branch_exception_type"]=originalException;
            }
        }

        [System.STAThread]
        private static int Main(string[] arguments) {
            if (arguments.Length==1 && arguments[0]=="--owned-fast-exit") return 7;
            if (arguments.Length==1 && arguments[0]=="--owned-live-identity") {
                System.Console.WriteLine("READY"); System.Console.Out.Flush();
                using (var reader=ReleaseReader(System.Console.OpenStandardInput())) {
                    var release=reader.ReadLineAsync();
                    return release.Wait(5000) && release.Result=="RELEASE" ? 0 : 8;
                }
            }
            MenuQaOptions options=null;
            var serializer=new System.Web.Script.Serialization.JavaScriptSerializer();
            var result=new System.Collections.Generic.Dictionary<string,object>();
            try {
                stage="parse_options";
                options=MenuQaOptions.Parse(arguments,true);
                Core(options,result);
                if (!options.CoreOnly) Full(options,result);
                stage="complete";
                result["status"]=options.CoreOnly?"core_only_passed":"passed";
                result["full_self_test_passed"]=!options.CoreOnly;
                result["scope"]=options.CoreOnly?"Pure identity, deferred callback and frame/path guards only; no Form or product process.":"Clean-source owned process identity and unshown Form controlled-failure regression.";
                result["source_sha"]=options.Source;
                result["binary_sha256"]=options.Hash;
                result["qt_menus_opened"]=0;
                result["product_binary_executed"]=false;
                result["user_input_injected"]=false;
                result["stage"]=stage;
                System.IO.Directory.CreateDirectory(options.Output);
                string json=serializer.Serialize(result);
                System.IO.File.WriteAllText(System.IO.Path.Combine(options.Output,"fixture-result.json"),json);
                System.Console.WriteLine(json);
                return 0;
            } catch (System.Exception error) {
                result["status"]="failed";
                result["full_self_test_passed"]=false;
                result["exception_type"]=error.GetType().FullName;
                result["stage"]=stage;
                var validation=activeValidationOptions??options;
                result["validation_stage"]=validation==null?null:validation.ValidationStage;
                var assertion=error as FixtureAssertionException;
                var binding=error as MenuQaBindingException;
                result["failure_code"]=assertion!=null?assertion.Code:binding!=null?binding.Code:"unexpected_exception";
                result["qt_menus_opened"]=0;
                result["product_binary_executed"]=false;
                string json=serializer.Serialize(result);
                if (options!=null) {
                    System.IO.Directory.CreateDirectory(options.Output);
                    System.IO.File.WriteAllText(System.IO.Path.Combine(options.Output,"fixture-result.json"),json);
                }
                System.Console.Error.WriteLine(json);
                return 1;
            }
        }
    }
}
