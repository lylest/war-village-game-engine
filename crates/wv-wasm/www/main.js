import * as THREE from "three";

// ---------- WASM init ----------
import init, { WasmGame, available_fighters, arena_bounds } from "../../pkg/wv_wasm.js";

await init();

const fighters = available_fighters();
console.log("Available fighters:", fighters);

const bounds = arena_bounds();
console.log("Arena bounds:", bounds);

const game = new WasmGame("Kael", "Knight");

// ---------- Three.js scene ----------
const scene = new THREE.Scene();
scene.background = new THREE.Color(0x1a1a2e);
scene.fog = new THREE.Fog(0x1a1a2e, 20, 40);

const camera = new THREE.PerspectiveCamera(50, innerWidth / innerHeight, 0.1, 100);
camera.position.set(0, 5, 14);
camera.lookAt(0, 1.2, 0);

const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setSize(innerWidth, innerHeight);
renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
renderer.shadowMap.enabled = true;
document.body.appendChild(renderer.domElement);

// Lights
const ambient = new THREE.AmbientLight(0x404060, 1.2);
scene.add(ambient);
const dirLight = new THREE.DirectionalLight(0xffeedd, 1.5);
dirLight.position.set(5, 10, 7);
dirLight.castShadow = true;
dirLight.shadow.mapSize.set(1024, 1024);
scene.add(dirLight);

// Arena floor
const floorGeo = new THREE.PlaneGeometry(
  bounds.max_x - bounds.min_x,
  bounds.max_z - bounds.min_z
);
const floorMat = new THREE.MeshStandardMaterial({ color: 0x2a2a3e, roughness: 0.8 });
const floor = new THREE.Mesh(floorGeo, floorMat);
floor.rotation.x = -Math.PI / 2;
floor.position.y = 0;
floor.receiveShadow = true;
scene.add(floor);

// Arena boundary lines
const linesMat = new THREE.LineBasicMaterial({ color: 0x444466 });
const linesGeo = new THREE.BufferGeometry().setFromPoints([
  new THREE.Vector3(bounds.min_x, 0.01, bounds.min_z),
  new THREE.Vector3(bounds.max_x, 0.01, bounds.min_z),
  new THREE.Vector3(bounds.max_x, 0.01, bounds.max_z),
  new THREE.Vector3(bounds.min_x, 0.01, bounds.max_z),
  new THREE.Vector3(bounds.min_x, 0.01, bounds.min_z),
]);
scene.add(new THREE.Line(linesGeo, linesMat));

// Fighter meshes (box placeholders)
function makeFighter(color) {
  const group = new THREE.Group();

  // Body
  const bodyGeo = new THREE.BoxGeometry(0.6, 1.4, 0.4);
  const bodyMat = new THREE.MeshStandardMaterial({ color });
  const body = new THREE.Mesh(bodyGeo, bodyMat);
  body.position.y = 1.0;
  body.castShadow = true;
  group.add(body);

  // Head
  const headGeo = new THREE.BoxGeometry(0.35, 0.35, 0.35);
  const headMat = new THREE.MeshStandardMaterial({ color: 0xddccaa });
  const head = new THREE.Mesh(headGeo, headMat);
  head.position.y = 2.0;
  head.castShadow = true;
  group.add(head);

  group.userData = { body, head, bodyMat };
  return group;
}

const p1Mesh = makeFighter(0x4488ff);
const p2Mesh = makeFighter(0xff4444);
scene.add(p1Mesh);
scene.add(p2Mesh);

// State colors for fighter body
const STATE_COLORS = {
  Idle: null,        // use default
  Moving: null,
  Attacking: 0xffaa00,
  Blocking: 0x00aaff,
  Dashing: 0x88ffaa,
  HitStun: 0xff0000,
  Airborne: 0xffff44,
  Knockdown: 0x884400,
  "Getting Up": 0x886644,
};

// ---------- Keyboard ----------
const keys = {};
addEventListener("keydown", (e) => { keys[e.code] = true; });
addEventListener("keyup", (e) => { keys[e.code] = false; });

function readInputs() {
  // P1: WASD + J/K/L/Shift/Space
  const p1_fwd   = !!keys["KeyD"];
  const p1_back  = !!keys["KeyA"];
  const p1_left  = !!keys["KeyW"];
  const p1_right = !!keys["KeyS"];
  const p1_light = !!keys["KeyJ"];
  const p1_heavy = !!keys["KeyK"];
  const p1_spec  = !!keys["KeyL"];
  const p1_block = !!keys["ShiftLeft"];
  const p1_dash  = !!keys["Space"];

  // P2: Arrows + Numpad 1/2/3/0/Enter
  const p2_fwd   = !!keys["ArrowRight"];
  const p2_back  = !!keys["ArrowLeft"];
  const p2_left  = !!keys["ArrowUp"];
  const p2_right = !!keys["ArrowDown"];
  const p2_light = !!keys["Numpad1"];
  const p2_heavy = !!keys["Numpad2"];
  const p2_spec  = !!keys["Numpad3"];
  const p2_block = !!keys["Numpad0"];
  const p2_dash  = !!keys["NumpadEnter"];

  // Pack into u32 bitflags
  let bits = 0;
  if (p1_fwd)   bits |= (1 << 0);
  if (p1_back)  bits |= (1 << 1);
  if (p1_left)  bits |= (1 << 2);
  if (p1_right) bits |= (1 << 3);
  if (p1_light) bits |= (1 << 4);
  if (p1_heavy) bits |= (1 << 5);
  if (p1_spec)  bits |= (1 << 6);
  if (p1_block) bits |= (1 << 7);
  if (p1_dash)  bits |= (1 << 8);

  if (p2_fwd)   bits |= (1 << 9);
  if (p2_back)  bits |= (1 << 10);
  if (p2_left)  bits |= (1 << 11);
  if (p2_right) bits |= (1 << 12);
  if (p2_light) bits |= (1 << 13);
  if (p2_heavy) bits |= (1 << 14);
  if (p2_spec)  bits |= (1 << 15);
  if (p2_block) bits |= (1 << 16);
  if (p2_dash)  bits |= (1 << 17);

  return bits;
}

// ---------- HUD elements ----------
const el = {
  p1Name: document.getElementById("p1-name"),
  p1Health: document.getElementById("p1-health"),
  p1Stamina: document.getElementById("p1-stamina"),
  p1Wins: document.getElementById("p1-wins"),
  p2Name: document.getElementById("p2-name"),
  p2Health: document.getElementById("p2-health"),
  p2Stamina: document.getElementById("p2-stamina"),
  p2Wins: document.getElementById("p2-wins"),
  round: document.getElementById("round-display"),
  timer: document.getElementById("timer-display"),
  phase: document.getElementById("phase-display"),
  hitInfo: document.getElementById("hit-info"),
};

let lastHitInfo = null;
let hitInfoTimer = 0;

function updateHUD(snap) {
  const f1 = snap.fighters[0];
  const f2 = snap.fighters[1];

  el.p1Name.textContent = `P1 ${f1.fighter_id} (${f1.weapon_type})`;
  el.p1Health.style.width = `${f1.health_pct * 100}%`;
  el.p1Stamina.style.width = `${f1.stamina_pct * 100}%`;
  el.p1Wins.textContent = "\u2605".repeat(f1.round_wins);

  el.p2Name.textContent = `P2 ${f2.fighter_id} (${f2.weapon_type})`;
  el.p2Health.style.width = `${f2.health_pct * 100}%`;
  el.p2Stamina.style.width = `${f2.stamina_pct * 100}%`;
  el.p2Wins.textContent = "\u2605".repeat(f2.round_wins);

  el.round.textContent = `Round ${snap.current_round}`;
  el.timer.textContent = snap.round_timer.toFixed(1);

  // Phase overlay
  if (snap.phase === "Countdown") {
    el.phase.textContent = snap.countdown_display;
    el.phase.style.display = "block";
  } else if (snap.phase === "RoundOver") {
    el.phase.textContent = "ROUND OVER";
    el.phase.style.display = "block";
  } else if (snap.phase === "MatchOver") {
    const winnerIdx = snap.winner;
    const winnerName = winnerIdx != null ? snap.fighters[winnerIdx].fighter_id : "Draw";
    el.phase.textContent = `${winnerName} WINS!`;
    el.phase.style.display = "block";
  } else {
    el.phase.style.display = "none";
  }

  // Hit info
  if (snap.last_hit_info && snap.last_hit_info !== lastHitInfo) {
    lastHitInfo = snap.last_hit_info;
    hitInfoTimer = 120; // show for 2 seconds
    el.hitInfo.textContent = lastHitInfo;
    el.hitInfo.style.opacity = "1";
  }
  if (hitInfoTimer > 0) {
    hitInfoTimer--;
    if (hitInfoTimer === 0) {
      el.hitInfo.style.opacity = "0";
    }
  }

  // Color health bar based on remaining health
  if (f1.health_pct < 0.25) {
    el.p1Health.style.background = "linear-gradient(180deg, #f44, #822)";
  } else if (f1.health_pct < 0.5) {
    el.p1Health.style.background = "linear-gradient(180deg, #fc4, #862)";
  } else {
    el.p1Health.style.background = "";
  }

  if (f2.health_pct < 0.25) {
    el.p2Health.style.background = "linear-gradient(180deg, #f44, #822)";
  } else if (f2.health_pct < 0.5) {
    el.p2Health.style.background = "linear-gradient(180deg, #fc4, #862)";
  } else {
    el.p2Health.style.background = "";
  }
}

// ---------- Update fighter meshes ----------
function updateFighterMesh(mesh, fighter) {
  mesh.position.set(fighter.position.x, fighter.position.y, fighter.position.z);

  // Face the right direction
  mesh.rotation.y = fighter.facing === "Right" ? 0 : Math.PI;

  // State-based body color
  const defaultColor = mesh === p1Mesh ? 0x4488ff : 0xff4444;
  const stateColor = STATE_COLORS[fighter.state];
  mesh.userData.bodyMat.color.setHex(stateColor != null ? stateColor : defaultColor);

  // Squash/stretch for visual feedback
  const body = mesh.userData.body;
  if (fighter.state === "Dashing") {
    body.scale.set(1.3, 0.8, 1);
  } else if (fighter.state === "Attacking" && fighter.attack && fighter.attack.phase === "Active") {
    body.scale.set(1.1, 1.05, 1.1);
  } else if (fighter.state === "HitStun") {
    body.scale.set(0.9, 1.1, 0.9);
  } else if (fighter.state === "Knockdown") {
    body.scale.set(1.2, 0.5, 1);
  } else {
    body.scale.set(1, 1, 1);
  }
}

// ---------- Game loop (fixed 60fps timestep) ----------
const TICK_MS = 1000 / 60;
let accumulator = 0;
let lastTime = performance.now();
let latestSnap = game.get_snapshot();

function loop(now) {
  requestAnimationFrame(loop);

  const dt = Math.min(now - lastTime, 100); // cap to avoid spiral of death
  lastTime = now;
  accumulator += dt;

  while (accumulator >= TICK_MS) {
    const bits = readInputs();
    latestSnap = game.tick_packed(bits);
    accumulator -= TICK_MS;
  }

  // Render with latest snapshot
  updateFighterMesh(p1Mesh, latestSnap.fighters[0]);
  updateFighterMesh(p2Mesh, latestSnap.fighters[1]);
  updateHUD(latestSnap);

  renderer.render(scene, camera);
}

requestAnimationFrame(loop);

// Resize handler
addEventListener("resize", () => {
  camera.aspect = innerWidth / innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(innerWidth, innerHeight);
});
