let r;const h=new Array(128).fill(void 0);h.push(void 0,null,!0,!1);function s(_){return h[_]}let E=h.length;function P(_){_<132||(h[_]=E,E=_)}function g(_){const t=s(_);return P(_),t}function w(_){E===h.length&&h.push(h.length+1);const t=E;return E=h[t],h[t]=_,t}const D=typeof TextDecoder<"u"?new TextDecoder("utf-8",{ignoreBOM:!0,fatal:!0}):{decode:()=>{throw Error("TextDecoder not available")}};typeof TextDecoder<"u"&&D.decode();let A=null;function W(){return(A===null||A.byteLength===0)&&(A=new Uint8Array(r.memory.buffer)),A}function v(_,t){return _=_>>>0,D.decode(W().subarray(_,_+t))}let d=0;function O(_,t){const e=t(_.length*1,1)>>>0;return W().set(_,e/1),d=_.length,e}let j=null;function c(){return(j===null||j.byteLength===0)&&(j=new Int32Array(r.memory.buffer)),j}const M=typeof TextEncoder<"u"?new TextEncoder("utf-8"):{encode:()=>{throw Error("TextEncoder not available")}},Y=typeof M.encodeInto=="function"?function(_,t){return M.encodeInto(_,t)}:function(_,t){const e=M.encode(_);return t.set(e),{read:_.length,written:e.length}};function T(_,t,e){if(e===void 0){const b=M.encode(_),f=t(b.length,1)>>>0;return W().subarray(f,f+b.length).set(b),d=b.length,f}let n=_.length,o=t(n,1)>>>0;const i=W();let a=0;for(;a<n;a++){const b=_.charCodeAt(a);if(b>127)break;i[o+a]=b}if(a!==n){a!==0&&(_=_.slice(a)),o=e(o,n,n=a+_.length*3,1)>>>0;const b=W().subarray(o+a,o+n),f=Y(_,b);a+=f.written}return d=a,o}function $(_,t){if(!(_ instanceof t))throw new Error(`expected instance of ${t.name}`);return _.ptr}function u(_){return _==null}function p(_,t){try{return _.apply(this,t)}catch(e){r.__wbindgen_exn_store(w(e))}}class L{static __wrap(t){t=t>>>0;const e=Object.create(L.prototype);return e.__wbg_ptr=t,e}__destroy_into_raw(){const t=this.__wbg_ptr;return this.__wbg_ptr=0,t}free(){const t=this.__destroy_into_raw();r.__wbg_draws_free(t)}get num_iterations(){return r.__wbg_get_draws_num_iterations(this.__wbg_ptr)>>>0}set num_iterations(t){r.__wbg_set_draws_num_iterations(this.__wbg_ptr,t)}get gut_shot(){return r.__wbg_get_draws_gut_shot(this.__wbg_ptr)>>>0}set gut_shot(t){r.__wbg_set_draws_gut_shot(this.__wbg_ptr,t)}get str8_draw(){return r.__wbg_get_draws_str8_draw(this.__wbg_ptr)>>>0}set str8_draw(t){r.__wbg_set_draws_str8_draw(this.__wbg_ptr,t)}get flush_draw(){return r.__wbg_get_draws_flush_draw(this.__wbg_ptr)>>>0}set flush_draw(t){r.__wbg_set_draws_flush_draw(this.__wbg_ptr,t)}get backdoor_flush_draw(){return r.__wbg_get_draws_backdoor_flush_draw(this.__wbg_ptr)>>>0}set backdoor_flush_draw(t){r.__wbg_set_draws_backdoor_flush_draw(this.__wbg_ptr,t)}get one_overcard(){return r.__wbg_get_draws_one_overcard(this.__wbg_ptr)>>>0}set one_overcard(t){r.__wbg_set_draws_one_overcard(this.__wbg_ptr,t)}get two_overcards(){return r.__wbg_get_draws_two_overcards(this.__wbg_ptr)>>>0}set two_overcards(t){r.__wbg_set_draws_two_overcards(this.__wbg_ptr,t)}get lo_paired(){return r.__wbg_get_draws_lo_paired(this.__wbg_ptr)>>>0}set lo_paired(t){r.__wbg_set_draws_lo_paired(this.__wbg_ptr,t)}get hi_paired(){return r.__wbg_get_draws_hi_paired(this.__wbg_ptr)>>>0}set hi_paired(t){r.__wbg_set_draws_hi_paired(this.__wbg_ptr,t)}get pp_paired(){return r.__wbg_get_draws_pp_paired(this.__wbg_ptr)>>>0}set pp_paired(t){r.__wbg_set_draws_pp_paired(this.__wbg_ptr,t)}static new(){const t=r.draws_new();return L.__wrap(t)}}class z{static __wrap(t){t=t>>>0;const e=Object.create(z.prototype);return e.__wbg_ptr=t,e}__destroy_into_raw(){const t=this.__wbg_ptr;return this.__wbg_ptr=0,t}free(){const t=this.__destroy_into_raw();r.__wbg_flopsimulationresults_free(t)}get_perc_family(t,e,n,o){return r.flopsimulationresults_get_perc_family(this.__wbg_ptr,!u(t),u(t)?0:t,e,n,o)}get_perc_family_or_better(t,e,n,o){return r.flopsimulationresults_get_perc_family_or_better(this.__wbg_ptr,!u(t),u(t)?0:t,e,n,o)}get_equity(t,e){return r.flopsimulationresults_get_equity(this.__wbg_ptr,!u(t),u(t)?0:t,e)}get_range_equity(t,e){try{const a=r.__wbindgen_add_to_stack_pointer(-16);r.flopsimulationresults_get_range_equity(a,this.__wbg_ptr,!u(t),u(t)?0:t,e);var n=c()[a/4+0],o=c()[a/4+1],i=c()[a/4+2];if(i)throw g(o);return g(n)}finally{r.__wbindgen_add_to_stack_pointer(16)}}get_range_it_count(t,e){try{const a=r.__wbindgen_add_to_stack_pointer(-16);r.flopsimulationresults_get_range_it_count(a,this.__wbg_ptr,!u(t),u(t)?0:t,e);var n=c()[a/4+0],o=c()[a/4+1],i=c()[a/4+2];if(i)throw g(o);return g(n)}finally{r.__wbindgen_add_to_stack_pointer(16)}}get_street_draw(t,e){try{const a=r.__wbindgen_add_to_stack_pointer(-16);r.flopsimulationresults_get_street_draw(a,this.__wbg_ptr,!u(t),u(t)?0:t,e);var n=c()[a/4+0],o=c()[a/4+1],i=c()[a/4+2];if(i)throw g(o);return g(n)}finally{r.__wbindgen_add_to_stack_pointer(16)}}get_num_players(){return r.flopsimulationresults_get_num_players(this.__wbg_ptr)>>>0}get_player_index(t){return r.flopsimulationresults_get_player_index(this.__wbg_ptr,t)>>>0}}class C{static __wrap(t){t=t>>>0;const e=Object.create(C.prototype);return e.__wbg_ptr=t,e}__destroy_into_raw(){const t=this.__wbg_ptr;return this.__wbg_ptr=0,t}free(){const t=this.__destroy_into_raw();r.__wbg_playerflopresults_free(t)}static new(){const t=r.playerflopresults_new();return C.__wrap(t)}}class K{__destroy_into_raw(){const t=this.__wbg_ptr;return this.__wbg_ptr=0,t}free(){const t=this.__destroy_into_raw();r.__wbg_preflopplayerinfo_free(t)}}class F{static __wrap(t){t=t>>>0;const e=Object.create(F.prototype);return e.__wbg_ptr=t,e}__destroy_into_raw(){const t=this.__wbg_ptr;return this.__wbg_ptr=0,t}free(){const t=this.__destroy_into_raw();r.__wbg_flop_analyzer_free(t)}static new(){const t=r.flop_analyzer_new();return F.__wrap(t)}set_board_cards(t){try{const o=r.__wbindgen_add_to_stack_pointer(-16),i=O(t,r.__wbindgen_malloc),a=d;r.flop_analyzer_set_board_cards(o,this.__wbg_ptr,i,a);var e=c()[o/4+0],n=c()[o/4+1];if(n)throw g(e)}finally{r.__wbindgen_add_to_stack_pointer(16)}}set_player_cards(t,e){try{const i=r.__wbindgen_add_to_stack_pointer(-16),a=O(e,r.__wbindgen_malloc),b=d;r.flop_analyzer_set_player_cards(i,this.__wbg_ptr,t,a,b);var n=c()[i/4+0],o=c()[i/4+1];if(o)throw g(n)}finally{r.__wbindgen_add_to_stack_pointer(16)}}set_player_range(t,e){try{const i=r.__wbindgen_add_to_stack_pointer(-16),a=T(e,r.__wbindgen_malloc,r.__wbindgen_realloc),b=d;r.flop_analyzer_set_player_range(i,this.__wbg_ptr,t,a,b);var n=c()[i/4+0],o=c()[i/4+1];if(o)throw g(n)}finally{r.__wbindgen_add_to_stack_pointer(16)}}set_player_state(t,e){r.flop_analyzer_set_player_state(this.__wbg_ptr,t,e)}clear_player_cards(t){r.flop_analyzer_clear_player_cards(this.__wbg_ptr,t)}reset(){r.flop_analyzer_reset(this.__wbg_ptr)}build_results(){const t=r.flop_analyzer_build_results(this.__wbg_ptr);return z.__wrap(t)}simulate_flop(t,e,n){try{const f=r.__wbindgen_add_to_stack_pointer(-16);$(e,z);var o=e.__destroy_into_raw();r.flop_analyzer_simulate_flop(f,this.__wbg_ptr,t,o,n);var i=c()[f/4+0],a=c()[f/4+1],b=c()[f/4+2];if(b)throw g(a);return z.__wrap(i)}finally{r.__wbindgen_add_to_stack_pointer(16)}}narrow_range(t,e,n,o,i){let a,b;try{const m=r.__wbindgen_add_to_stack_pointer(-16),U=T(t,r.__wbindgen_malloc,r.__wbindgen_realloc),I=d,q=T(e,r.__wbindgen_malloc,r.__wbindgen_realloc),V=d,x=O(o,r.__wbindgen_malloc),H=d;r.flop_analyzer_narrow_range(m,this.__wbg_ptr,U,I,q,V,n,x,H,i);var f=c()[m/4+0],R=c()[m/4+1],S=c()[m/4+2],k=c()[m/4+3],y=f,l=R;if(k)throw y=0,l=0,g(S);return a=y,b=l,v(y,l)}finally{r.__wbindgen_add_to_stack_pointer(16),r.__wbindgen_free(a,b,1)}}narrow_range_by_pref(t,e,n,o){let i,a;try{const l=r.__wbindgen_add_to_stack_pointer(-16),m=T(t,r.__wbindgen_malloc,r.__wbindgen_realloc),U=d,I=O(n,r.__wbindgen_malloc),q=d;r.flop_analyzer_narrow_range_by_pref(l,this.__wbg_ptr,m,U,e,I,q,o);var b=c()[l/4+0],f=c()[l/4+1],R=c()[l/4+2],S=c()[l/4+3],k=b,y=f;if(S)throw k=0,y=0,g(R);return i=k,a=y,v(k,y)}finally{r.__wbindgen_add_to_stack_pointer(16),r.__wbindgen_free(i,a,1)}}}async function G(_,t){if(typeof Response=="function"&&_ instanceof Response){if(typeof WebAssembly.instantiateStreaming=="function")try{return await WebAssembly.instantiateStreaming(_,t)}catch(n){if(_.headers.get("Content-Type")!="application/wasm")console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n",n);else throw n}const e=await _.arrayBuffer();return await WebAssembly.instantiate(e,t)}else{const e=await WebAssembly.instantiate(_,t);return e instanceof WebAssembly.Instance?{instance:e,module:_}:e}}function B(){const _={};return _.wbg={},_.wbg.__wbindgen_object_drop_ref=function(t){g(t)},_.wbg.__wbindgen_object_clone_ref=function(t){const e=s(t);return w(e)},_.wbg.__wbindgen_is_object=function(t){const e=s(t);return typeof e=="object"&&e!==null},_.wbg.__wbindgen_number_new=function(t){return w(t)},_.wbg.__wbindgen_string_new=function(t,e){const n=v(t,e);return w(n)},_.wbg.__wbg_set_9182712abebf82ef=function(t,e,n){s(t)[g(e)]=g(n)},_.wbg.__wbg_debug_678fc976919895d2=function(t,e,n,o){console.debug(s(t),s(e),s(n),s(o))},_.wbg.__wbg_error_e60eff06f24ab7a4=function(t){console.error(s(t))},_.wbg.__wbg_error_ce00188b70015ed4=function(t,e,n,o){console.error(s(t),s(e),s(n),s(o))},_.wbg.__wbg_info_7904cb81904ea2ec=function(t,e,n,o){console.info(s(t),s(e),s(n),s(o))},_.wbg.__wbg_log_aaedbaa276606939=function(t,e,n,o){console.log(s(t),s(e),s(n),s(o))},_.wbg.__wbg_warn_0345511f899411e2=function(t,e,n,o){console.warn(s(t),s(e),s(n),s(o))},_.wbg.__wbg_new_abda76e883ba8a5f=function(){const t=new Error;return w(t)},_.wbg.__wbg_stack_658279fe44541cf6=function(t,e){const n=s(e).stack,o=T(n,r.__wbindgen_malloc,r.__wbindgen_realloc),i=d;c()[t/4+1]=i,c()[t/4+0]=o},_.wbg.__wbg_error_f851667af71bcfc6=function(t,e){let n,o;try{n=t,o=e,console.error(v(t,e))}finally{r.__wbindgen_free(n,o,1)}},_.wbg.__wbg_crypto_58f13aa23ffcb166=function(t){const e=s(t).crypto;return w(e)},_.wbg.__wbg_process_5b786e71d465a513=function(t){const e=s(t).process;return w(e)},_.wbg.__wbg_versions_c2ab80650590b6a2=function(t){const e=s(t).versions;return w(e)},_.wbg.__wbg_node_523d7bd03ef69fba=function(t){const e=s(t).node;return w(e)},_.wbg.__wbindgen_is_string=function(t){return typeof s(t)=="string"},_.wbg.__wbg_msCrypto_abcb1295e768d1f2=function(t){const e=s(t).msCrypto;return w(e)},_.wbg.__wbg_require_2784e593a4674877=function(){return p(function(){const t=module.require;return w(t)},arguments)},_.wbg.__wbindgen_is_function=function(t){return typeof s(t)=="function"},_.wbg.__wbg_randomFillSync_a0d98aa11c81fe89=function(){return p(function(t,e){s(t).randomFillSync(g(e))},arguments)},_.wbg.__wbg_getRandomValues_504510b5564925af=function(){return p(function(t,e){s(t).getRandomValues(s(e))},arguments)},_.wbg.__wbg_new_ffc6d4d085022169=function(){const t=new Array;return w(t)},_.wbg.__wbg_newnoargs_c62ea9419c21fbac=function(t,e){const n=new Function(v(t,e));return w(n)},_.wbg.__wbg_call_90c26b09837aba1c=function(){return p(function(t,e){const n=s(t).call(s(e));return w(n)},arguments)},_.wbg.__wbg_new_9fb8d994e1c0aaac=function(){const t=new Object;return w(t)},_.wbg.__wbg_self_f0e34d89f33b99fd=function(){return p(function(){const t=self.self;return w(t)},arguments)},_.wbg.__wbg_window_d3b084224f4774d7=function(){return p(function(){const t=window.window;return w(t)},arguments)},_.wbg.__wbg_globalThis_9caa27ff917c6860=function(){return p(function(){const t=globalThis.globalThis;return w(t)},arguments)},_.wbg.__wbg_global_35dfdd59a4da3e74=function(){return p(function(){const t=global.global;return w(t)},arguments)},_.wbg.__wbindgen_is_undefined=function(t){return s(t)===void 0},_.wbg.__wbg_set_f2740edb12e318cd=function(t,e,n){s(t)[e>>>0]=g(n)},_.wbg.__wbg_new_a64e3f2afc2cf2f8=function(t,e){const n=new Error(v(t,e));return w(n)},_.wbg.__wbg_call_5da1969d7cd31ccd=function(){return p(function(t,e,n){const o=s(t).call(s(e),s(n));return w(o)},arguments)},_.wbg.__wbg_buffer_a448f833075b71ba=function(t){const e=s(t).buffer;return w(e)},_.wbg.__wbg_newwithbyteoffsetandlength_d0482f893617af71=function(t,e,n){const o=new Uint8Array(s(t),e>>>0,n>>>0);return w(o)},_.wbg.__wbg_new_8f67e318f15d7254=function(t){const e=new Uint8Array(s(t));return w(e)},_.wbg.__wbg_set_2357bf09366ee480=function(t,e,n){s(t).set(s(e),n>>>0)},_.wbg.__wbg_newwithlength_6c2df9e2f3028c43=function(t){const e=new Uint8Array(t>>>0);return w(e)},_.wbg.__wbg_subarray_2e940e41c0f5a1d9=function(t,e,n){const o=s(t).subarray(e>>>0,n>>>0);return w(o)},_.wbg.__wbindgen_throw=function(t,e){throw new Error(v(t,e))},_.wbg.__wbindgen_memory=function(){const t=r.memory;return w(t)},_}function N(_,t){return r=_.exports,J.__wbindgen_wasm_module=t,j=null,A=null,r}function Q(_){if(r!==void 0)return r;const t=B();_ instanceof WebAssembly.Module||(_=new WebAssembly.Module(_));const e=new WebAssembly.Instance(_,t);return N(e,_)}async function J(_){if(r!==void 0)return r;typeof _>"u"&&(_=new URL(""+new URL("poker_eval_bg-nCTY_BtN.wasm",import.meta.url).href,import.meta.url));const t=B();(typeof _=="string"||typeof Request=="function"&&_ instanceof Request||typeof URL=="function"&&_ instanceof URL)&&(_=fetch(_));const{instance:e,module:n}=await G(await _,t);return N(e,n)}export{L as Draws,z as FlopSimulationResults,C as PlayerFlopResults,K as PreflopPlayerInfo,J as default,F as flop_analyzer,Q as initSync};