const {createRequire}=require('node:module'),path=require('node:path'),fs=require('node:fs'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');const req=createRequire(root+'/integration/renderer/package.json');const Ajv=req('ajv/dist/2020.js').default;
const schema=JSON.parse(fs.readFileSync(root+'/fixtures/beta/schemas/extension-v2.schema.json'));const manifest=JSON.parse(fs.readFileSync(root+'/manifest.json'));
const ajv=new Ajv({strict:false,allErrors:true});const validate=ajv.compile(schema);assert(validate(manifest),JSON.stringify(validate.errors));
const mutations=[m=>m.author='unsupported',m=>m.runtime='javascript',m=>m.schemaVersion=1,m=>m.id='raycast.ascii',m=>m.icon='assets/icon.png',m=>m.replaces=[],m=>m.commands[0].platforms=['macos'],m=>m.commands[0].runtime='wasm',m=>m.verification[0].state='verified',m=>delete m.verification,m=>m.commands=[],m=>m.preferences='oops'];
for(const mutate of mutations){const m=structuredClone(manifest);mutate(m);assert(!validate(m),'negative schema case passed');}
assert.equal(manifest.commands[0].preferences[0].name,'defaultWrapLatex');assert.equal(manifest.commands[0].preferences[0].default,'single');assert.deepEqual(manifest.verification.map(v=>v.state),['not-run','not-run']);console.log(`Draft 2020-12 schema: valid manifest + ${mutations.length} negative cases`);
