const {invoke}=window.__TAURI__.core;
let revision=0;
const output=document.getElementById("output");
async function show(action){try{const result=await action();if(typeof result.revision==="number")revision=result.revision;output.textContent=JSON.stringify(result,null,2);}catch(error){output.textContent=String(error);}}
document.getElementById("add").onclick=()=>show(()=>invoke("dispatch",{command:{kind:"add",expected_revision:revision,id:"cpu",name:"CPU"}}));
document.getElementById("toggle").onclick=()=>show(()=>invoke("dispatch",{command:{kind:"toggle",expected_revision:revision,id:"cpu"}}));
for(const action of ["undo","redo"])document.getElementById(action).onclick=()=>show(()=>invoke("dispatch",{command:{kind:action,expected_revision:revision}}));
const orderId=()=>document.getElementById("order").value;
document.getElementById("submit").onclick=()=>show(()=>invoke("create_order",{id:orderId(),units:2}));
document.getElementById("refresh").onclick=()=>show(()=>invoke("refresh_order",{id:orderId()}));
document.getElementById("metrics").onclick=()=>show(()=>invoke("metric_query"));
window.__TAURI__.event.listen("orders-event",event=>{document.getElementById("events").textContent=event.payload.disconnected?"Observer disconnected: refresh authoritative API.":"Order event observed: refresh authoritative API."});
show(()=>invoke("view"));