// Executes the immutable locked library; expectations never come from Rust.
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const path=require('node:path');const root=path.resolve(__dirname,'..');
let c={exports:{}};vm.runInNewContext(fs.readFileSync(root+'/fixtures/library/package/build/index.js','utf8')+'\nexports.symbols=AMsymbols;',c);
assert.equal(fs.readFileSync(root+'/src/symbols.json','utf8'),JSON.stringify(c.exports.symbols,null,2)+'\n');
const inputs=new Set(['',' ','\t\n','x^2 + y^2 = z^2','2=(((3-x)xx2)/(3-x))','sqrt(x)','root(3)(8)','x_(i)^2','x_1^','x_1^)','[(1,2),(3,4)]','[[a,b],[c,d]]','|x|','|','(',')','"','"hello world"','text(hi)','😀 + Қ','α+β','&nbsp;x&gt;y','a\\b','x/-y','sin^2 x','(x_1^)','sqrt(x_1^)','{x_1^}', 'x\u2003y','\ufeffx','x\u0000y','a\nb','2.35','0.5','.5','5.','text( leading )','color(red)(x)','{:a,b:}','[[1],[2]]','[(1,2),(3)]']);
for(const s of c.exports.symbols)for(const tail of ['', ' x','(a,b)','_1^2','(x)/(y)'])inputs.add(s.input+tail);
let seed=73;const rnd=n=>{seed=(Math.imul(seed,1664525)+1013904223)>>>0;return seed%n};const atom=['x','2','alpha','oo','sqrt(x)','(a+b)','text(hi)','-x','😀','[x,y]'];
for(let i=0;i<2500;i++){let s='';for(let j=0,n=1+rnd(7);j<n;j++)s+=atom[rnd(atom.length)]+['',' ','+','/','^','_',',',')','('][rnd(9)];inputs.add(s);}
const rows=[...inputs].map(input=>{let output;try{output=c.exports.default(input)}catch{output='Error parsing expression'}return {input,outputUnits:Array.from({length:output.length},(_,i)=>output.charCodeAt(i))}});
const bytes=JSON.stringify(rows,null,2)+'\n',dest=root+'/fixtures/oracle.json';
if(process.argv.includes('--write'))fs.writeFileSync(dest,bytes);else assert.equal(fs.readFileSync(dest,'utf8'),bytes);
console.log(`Locked-library oracle: ${rows.length} expressions, ${c.exports.symbols.length} symbol entries`);
