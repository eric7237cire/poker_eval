let r;const d=new Array(128).fill(void 0);d.push(void 0,null,!0,!1);function o(_){return d[_]}let k=d.length;function U(_){_<132||(d[_]=k,k=_)}function g(_){const t=o(_);return U(_),t}function i(_){k===d.length&&d.push(d.length+1);const t=k;return k=d[t],d[t]=_,t}const z=typeof TextDecoder<"u"?new TextDecoder("utf-8",{ignoreBOM:!0,fatal:!0}):{decode:()=>{throw Error("TextDecoder not available")}};typeof TextDecoder<"u"&&z.decode();let y=null;function v(){return(y===null||y.byteLength===0)&&(y=new Uint8Array(r.memory.buffer)),y}function h(_,t){return _=_>>>0,z.decode(v().subarray(_,_+t))}let l=0;function O(_,t){const e=t(_.length*1,1)>>>0;return v().set(_,e/1),l=_.length,e}let m=null;function w(){return(m===null||m.byteLength===0)&&(m=new Int32Array(r.memory.buffer)),m}const j=typeof TextEncoder<"u"?new TextEncoder("utf-8"):{encode:()=>{throw Error("TextEncoder not available")}},I=typeof j.encodeInto=="function"?function(_,t){return j.encodeInto(_,t)}:function(_,t){const e=j.encode(_);return t.set(e),{read:_.length,written:e.length}};function M(_,t,e){if(e===void 0){const b=j.encode(_),p=t(b.length,1)>>>0;return v().subarray(p,p+b.length).set(b),l=b.length,p}let n=_.length,s=t(n,1)>>>0;const c=v();let a=0;for(;a<n;a++){const b=_.charCodeAt(a);if(b>127)break;c[s+a]=b}if(a!==n){a!==0&&(_=_.slice(a)),s=e(s,n,n=a+_.length*3,1)>>>0;const b=v().subarray(s+a,s+n),p=I(_,b);a+=p.written}return l=a,s}function q(_,t){if(!(_ instanceof t))throw new Error(`expected instance of ${t.name}`);return _.ptr}function f(_){return _==null}function u(_,t){try{return _.apply(this,t)}catch(e){r.__wbindgen_exn_store(i(e))}}class T{static __wrap(t){t=t>>>0;const e=Object.create(T.prototype);return e.__wbg_ptr=t,e}__destroy_into_raw(){const t=this.__wbg_ptr;return this.__wbg_ptr=0,t}free(){const t=this.__destroy_into_raw();r.__wbg_draws_free(t)}get num_iterations(){return r.__wbg_get_draws_num_iterations(this.__wbg_ptr)>>>0}set num_iterations(t){r.__wbg_set_draws_num_iterations(this.__wbg_ptr,t)}get gut_shot(){return r.__wbg_get_draws_gut_shot(this.__wbg_ptr)>>>0}set gut_shot(t){r.__wbg_set_draws_gut_shot(this.__wbg_ptr,t)}get str8_draw(){return r.__wbg_get_draws_str8_draw(this.__wbg_ptr)>>>0}set str8_draw(t){r.__wbg_set_draws_str8_draw(this.__wbg_ptr,t)}get flush_draw(){return r.__wbg_get_draws_flush_draw(this.__wbg_ptr)>>>0}set flush_draw(t){r.__wbg_set_draws_flush_draw(this.__wbg_ptr,t)}get backdoor_flush_draw(){return r.__wbg_get_draws_backdoor_flush_draw(this.__wbg_ptr)>>>0}set backdoor_flush_draw(t){r.__wbg_set_draws_backdoor_flush_draw(this.__wbg_ptr,t)}get one_overcard(){return r.__wbg_get_draws_one_overcard(this.__wbg_ptr)>>>0}set one_overcard(t){r.__wbg_set_draws_one_overcard(this.__wbg_ptr,t)}get two_overcards(){return r.__wbg_get_draws_two_overcards(this.__wbg_ptr)>>>0}set two_overcards(t){r.__wbg_set_draws_two_overcards(this.__wbg_ptr,t)}get lo_paired(){return r.__wbg_get_draws_lo_paired(this.__wbg_ptr)>>>0}set lo_paired(t){r.__wbg_set_draws_lo_paired(this.__wbg_ptr,t)}get hi_paired(){return r.__wbg_get_draws_hi_paired(this.__wbg_ptr)>>>0}set hi_paired(t){r.__wbg_set_draws_hi_paired(this.__wbg_ptr,t)}get pp_paired(){return r.__wbg_get_draws_pp_paired(this.__wbg_ptr)>>>0}set pp_paired(t){r.__wbg_set_draws_pp_paired(this.__wbg_ptr,t)}static new(){const t=r.draws_new();return T.__wrap(t)}}class A{static __wrap(t){t=t>>>0;const e=Object.create(A.prototype);return e.__wbg_ptr=t,e}__destroy_into_raw(){const t=this.__wbg_ptr;return this.__wbg_ptr=0,t}free(){const t=this.__destroy_into_raw();r.__wbg_flopsimulationresults_free(t)}get_perc_family(t,e,n){return r.flopsimulationresults_get_perc_family(this.__wbg_ptr,!f(t),f(t)?0:t,e,n)}get_perc_family_or_better(t,e,n){return r.flopsimulationresults_get_perc_family_or_better(this.__wbg_ptr,!f(t),f(t)?0:t,e,n)}get_equity(t,e){return r.flopsimulationresults_get_equity(this.__wbg_ptr,!f(t),f(t)?0:t,e)}get_range_equity(t,e){try{const a=r.__wbindgen_add_to_stack_pointer(-16);r.flopsimulationresults_get_range_equity(a,this.__wbg_ptr,!f(t),f(t)?0:t,e);var n=w()[a/4+0],s=w()[a/4+1],c=w()[a/4+2];if(c)throw g(s);return g(n)}finally{r.__wbindgen_add_to_stack_pointer(16)}}get_range_it_count(t,e){try{const a=r.__wbindgen_add_to_stack_pointer(-16);r.flopsimulationresults_get_range_it_count(a,this.__wbg_ptr,!f(t),f(t)?0:t,e);var n=w()[a/4+0],s=w()[a/4+1],c=w()[a/4+2];if(c)throw g(s);return g(n)}finally{r.__wbindgen_add_to_stack_pointer(16)}}get_street_draw(t,e){try{const a=r.__wbindgen_add_to_stack_pointer(-16);r.flopsimulationresults_get_street_draw(a,this.__wbg_ptr,!f(t),f(t)?0:t,e);var n=w()[a/4+0],s=w()[a/4+1],c=w()[a/4+2];if(c)throw g(s);return g(n)}finally{r.__wbindgen_add_to_stack_pointer(16)}}get_num_players(){return r.flopsimulationresults_get_num_players(this.__wbg_ptr)>>>0}get_player_index(t){return r.flopsimulationresults_get_player_index(this.__wbg_ptr,t)>>>0}}class E{static __wrap(t){t=t>>>0;const e=Object.create(E.prototype);return e.__wbg_ptr=t,e}__destroy_into_raw(){const t=this.__wbg_ptr;return this.__wbg_ptr=0,t}free(){const t=this.__destroy_into_raw();r.__wbg_playerflopresults_free(t)}static new(){const t=r.playerflopresults_new();return E.__wrap(t)}}class F{__destroy_into_raw(){const t=this.__wbg_ptr;return this.__wbg_ptr=0,t}free(){const t=this.__destroy_into_raw();r.__wbg_preflopplayerinfo_free(t)}}class W{static __wrap(t){t=t>>>0;const e=Object.create(W.prototype);return e.__wbg_ptr=t,e}__destroy_into_raw(){const t=this.__wbg_ptr;return this.__wbg_ptr=0,t}free(){const t=this.__destroy_into_raw();r.__wbg_flop_analyzer_free(t)}static new(){const t=r.flop_analyzer_new();return W.__wrap(t)}set_board_cards(t){try{const s=r.__wbindgen_add_to_stack_pointer(-16),c=O(t,r.__wbindgen_malloc),a=l;r.flop_analyzer_set_board_cards(s,this.__wbg_ptr,c,a);var e=w()[s/4+0],n=w()[s/4+1];if(n)throw g(e)}finally{r.__wbindgen_add_to_stack_pointer(16)}}set_player_cards(t,e){try{const c=r.__wbindgen_add_to_stack_pointer(-16),a=O(e,r.__wbindgen_malloc),b=l;r.flop_analyzer_set_player_cards(c,this.__wbg_ptr,t,a,b);var n=w()[c/4+0],s=w()[c/4+1];if(s)throw g(n)}finally{r.__wbindgen_add_to_stack_pointer(16)}}set_player_range(t,e){try{const c=r.__wbindgen_add_to_stack_pointer(-16),a=M(e,r.__wbindgen_malloc,r.__wbindgen_realloc),b=l;r.flop_analyzer_set_player_range(c,this.__wbg_ptr,t,a,b);var n=w()[c/4+0],s=w()[c/4+1];if(s)throw g(n)}finally{r.__wbindgen_add_to_stack_pointer(16)}}set_player_state(t,e){r.flop_analyzer_set_player_state(this.__wbg_ptr,t,e)}clear_player_cards(t){r.flop_analyzer_clear_player_cards(this.__wbg_ptr,t)}reset(){r.flop_analyzer_reset(this.__wbg_ptr)}build_results(){const t=r.flop_analyzer_build_results(this.__wbg_ptr);return A.__wrap(t)}simulate_flop(t,e){try{const b=r.__wbindgen_add_to_stack_pointer(-16);q(e,A);var n=e.__destroy_into_raw();r.flop_analyzer_simulate_flop(b,this.__wbg_ptr,t,n);var s=w()[b/4+0],c=w()[b/4+1],a=w()[b/4+2];if(a)throw g(c);return A.__wrap(s)}finally{r.__wbindgen_add_to_stack_pointer(16)}}}async function L(_,t){if(typeof Response=="function"&&_ instanceof Response){if(typeof WebAssembly.instantiateStreaming=="function")try{return await WebAssembly.instantiateStreaming(_,t)}catch(n){if(_.headers.get("Content-Type")!="application/wasm")console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n",n);else throw n}const e=await _.arrayBuffer();return await WebAssembly.instantiate(e,t)}else{const e=await WebAssembly.instantiate(_,t);return e instanceof WebAssembly.Instance?{instance:e,module:_}:e}}function R(){const _={};return _.wbg={},_.wbg.__wbindgen_object_drop_ref=function(t){g(t)},_.wbg.__wbindgen_object_clone_ref=function(t){const e=o(t);return i(e)},_.wbg.__wbindgen_is_object=function(t){const e=o(t);return typeof e=="object"&&e!==null},_.wbg.__wbindgen_number_new=function(t){return i(t)},_.wbg.__wbindgen_string_new=function(t,e){const n=h(t,e);return i(n)},_.wbg.__wbg_set_9182712abebf82ef=function(t,e,n){o(t)[g(e)]=g(n)},_.wbg.__wbg_debug_678fc976919895d2=function(t,e,n,s){console.debug(o(t),o(e),o(n),o(s))},_.wbg.__wbg_error_e60eff06f24ab7a4=function(t){console.error(o(t))},_.wbg.__wbg_error_ce00188b70015ed4=function(t,e,n,s){console.error(o(t),o(e),o(n),o(s))},_.wbg.__wbg_info_7904cb81904ea2ec=function(t,e,n,s){console.info(o(t),o(e),o(n),o(s))},_.wbg.__wbg_log_aaedbaa276606939=function(t,e,n,s){console.log(o(t),o(e),o(n),o(s))},_.wbg.__wbg_warn_0345511f899411e2=function(t,e,n,s){console.warn(o(t),o(e),o(n),o(s))},_.wbg.__wbg_new_abda76e883ba8a5f=function(){const t=new Error;return i(t)},_.wbg.__wbg_stack_658279fe44541cf6=function(t,e){const n=o(e).stack,s=M(n,r.__wbindgen_malloc,r.__wbindgen_realloc),c=l;w()[t/4+1]=c,w()[t/4+0]=s},_.wbg.__wbg_error_f851667af71bcfc6=function(t,e){let n,s;try{n=t,s=e,console.error(h(t,e))}finally{r.__wbindgen_free(n,s,1)}},_.wbg.__wbg_crypto_58f13aa23ffcb166=function(t){const e=o(t).crypto;return i(e)},_.wbg.__wbg_process_5b786e71d465a513=function(t){const e=o(t).process;return i(e)},_.wbg.__wbg_versions_c2ab80650590b6a2=function(t){const e=o(t).versions;return i(e)},_.wbg.__wbg_node_523d7bd03ef69fba=function(t){const e=o(t).node;return i(e)},_.wbg.__wbindgen_is_string=function(t){return typeof o(t)=="string"},_.wbg.__wbg_msCrypto_abcb1295e768d1f2=function(t){const e=o(t).msCrypto;return i(e)},_.wbg.__wbg_require_2784e593a4674877=function(){return u(function(){const t=module.require;return i(t)},arguments)},_.wbg.__wbindgen_is_function=function(t){return typeof o(t)=="function"},_.wbg.__wbg_randomFillSync_a0d98aa11c81fe89=function(){return u(function(t,e){o(t).randomFillSync(g(e))},arguments)},_.wbg.__wbg_getRandomValues_504510b5564925af=function(){return u(function(t,e){o(t).getRandomValues(o(e))},arguments)},_.wbg.__wbg_new_ffc6d4d085022169=function(){const t=new Array;return i(t)},_.wbg.__wbg_newnoargs_c62ea9419c21fbac=function(t,e){const n=new Function(h(t,e));return i(n)},_.wbg.__wbg_call_90c26b09837aba1c=function(){return u(function(t,e){const n=o(t).call(o(e));return i(n)},arguments)},_.wbg.__wbg_new_9fb8d994e1c0aaac=function(){const t=new Object;return i(t)},_.wbg.__wbg_self_f0e34d89f33b99fd=function(){return u(function(){const t=self.self;return i(t)},arguments)},_.wbg.__wbg_window_d3b084224f4774d7=function(){return u(function(){const t=window.window;return i(t)},arguments)},_.wbg.__wbg_globalThis_9caa27ff917c6860=function(){return u(function(){const t=globalThis.globalThis;return i(t)},arguments)},_.wbg.__wbg_global_35dfdd59a4da3e74=function(){return u(function(){const t=global.global;return i(t)},arguments)},_.wbg.__wbindgen_is_undefined=function(t){return o(t)===void 0},_.wbg.__wbg_set_f2740edb12e318cd=function(t,e,n){o(t)[e>>>0]=g(n)},_.wbg.__wbg_new_a64e3f2afc2cf2f8=function(t,e){const n=new Error(h(t,e));return i(n)},_.wbg.__wbg_call_5da1969d7cd31ccd=function(){return u(function(t,e,n){const s=o(t).call(o(e),o(n));return i(s)},arguments)},_.wbg.__wbg_buffer_a448f833075b71ba=function(t){const e=o(t).buffer;return i(e)},_.wbg.__wbg_newwithbyteoffsetandlength_d0482f893617af71=function(t,e,n){const s=new Uint8Array(o(t),e>>>0,n>>>0);return i(s)},_.wbg.__wbg_new_8f67e318f15d7254=function(t){const e=new Uint8Array(o(t));return i(e)},_.wbg.__wbg_set_2357bf09366ee480=function(t,e,n){o(t).set(o(e),n>>>0)},_.wbg.__wbg_newwithlength_6c2df9e2f3028c43=function(t){const e=new Uint8Array(t>>>0);return i(e)},_.wbg.__wbg_subarray_2e940e41c0f5a1d9=function(t,e,n){const s=o(t).subarray(e>>>0,n>>>0);return i(s)},_.wbg.__wbindgen_throw=function(t,e){throw new Error(h(t,e))},_.wbg.__wbindgen_memory=function(){const t=r.memory;return i(t)},_}function S(_,t){return r=_.exports,C.__wbindgen_wasm_module=t,m=null,y=null,r}function D(_){if(r!==void 0)return r;const t=R();_ instanceof WebAssembly.Module||(_=new WebAssembly.Module(_));const e=new WebAssembly.Instance(_,t);return S(e,_)}async function C(_){if(r!==void 0)return r;typeof _>"u"&&(_=new URL(""+new URL("poker_eval_bg-4HxCmE6f.wasm",import.meta.url).href,import.meta.url));const t=R();(typeof _=="string"||typeof Request=="function"&&_ instanceof Request||typeof URL=="function"&&_ instanceof URL)&&(_=fetch(_));const{instance:e,module:n}=await L(await _,t);return S(e,n)}export{T as Draws,A as FlopSimulationResults,E as PlayerFlopResults,F as PreflopPlayerInfo,C as default,W as flop_analyzer,D as initSync};