#!/usr/bin/env node

// This script fixes the generated declarations to use hardcoded canister IDs
// Run after dfx generate to ensure self-contained builds

import fs from 'fs';
import path from 'path';

const BACKEND_CANISTER_ID = "qakk3-qyaaa-aaaam-qd4ja-cai";
const LLM_CANISTER_ID = "w36hm-eqaaa-aaaal-qr76a-cai";

function fixBackendDeclaration() {
  const filePath = 'src/declarations/backend/index.js';
  if (!fs.existsSync(filePath)) {
    console.log('Backend declaration not found, skipping...');
    return;
  }

  let content = fs.readFileSync(filePath, 'utf8');
  
  // Replace the canisterId export with hardcoded value
  content = content.replace(
    /export const canisterId =[\s\S]*?;/m,
    `export const canisterId =
  // Hardcode permanent backend canister ID to ensure self-contained publish via ICP Ninja
  "${BACKEND_CANISTER_ID}";`
  );

  // Replace root key fetch logic
  content = content.replace(
    /\/\/ Fetch root key for certificate validation during development[\s\S]*?}\s*}/m,
    `// Fetch root key only when running locally (dfx replica). Avoid in mainnet/Ninja publish.
  const isLocal =
    typeof window !== "undefined" &&
    (window.location.hostname === "127.0.0.1" ||
      window.location.hostname === "localhost");
  if (isLocal) {
    agent.fetchRootKey().catch((err) => {
      console.warn(
        "Unable to fetch root key. Check to ensure that your local replica is running"
      );
      console.error(err);
    });
  }`
  );

  fs.writeFileSync(filePath, content);
  console.log('✅ Fixed backend declaration with hardcoded canister ID');
}

function fixLLMDeclaration() {
  const filePath = 'src/declarations/llm/index.js';
  if (!fs.existsSync(filePath)) {
    console.log('LLM declaration not found, skipping...');
    return;
  }

  let content = fs.readFileSync(filePath, 'utf8');
  
  // Replace the canisterId export with hardcoded value
  content = content.replace(
    /export const canisterId =[\s\S]*?;/m,
    `export const canisterId =
  // Hardcode permanent LLM canister ID to ensure self-contained publish via ICP Ninja
  "${LLM_CANISTER_ID}";`
  );

  // Replace root key fetch logic
  content = content.replace(
    /\/\/ Fetch root key for certificate validation during development[\s\S]*?}\s*}/m,
    `// Fetch root key only when running locally (dfx replica). Avoid in mainnet/Ninja publish.
  const isLocal =
    typeof window !== "undefined" &&
    (window.location.hostname === "127.0.0.1" ||
      window.location.hostname === "localhost");
  if (isLocal) {
    agent.fetchRootKey().catch((err) => {
      console.warn(
        "Unable to fetch root key. Check to ensure that your local replica is running"
      );
      console.error(err);
    });
  }`
  );

  fs.writeFileSync(filePath, content);
  console.log('✅ Fixed LLM declaration with hardcoded canister ID');
}

console.log('🔧 Fixing declarations with hardcoded canister IDs...');
fixBackendDeclaration();
fixLLMDeclaration();
console.log('✅ All declarations fixed for self-contained deployment');
