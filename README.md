# Disk Analyzer - Analýza obsadenosti úložiska

Desktopová aplikácia v Ruste na vizualizáciu obsadenosti diskového priestoru pomocou treemap grafov (štvorcové grafy).

## Vlastnosti

- **Treemap vizualizácia** - Štvorcové grafy kde veľkosť bloku reprezentuje veľkosť súboru/priečinka
- **Plne rekurzívne skenovanie** - Skenuje celý adresárový strom bez obmedzenia hĺbky
- **Interaktívna navigácia** - Dvojklik na priečinok pre vstup dovnútra, tlačidlá "Späť" a "Koreň"
- **Presné veľkosti** - Veľkosť priečinka = súčet všetkých súborov v ňom (rekurzívne)
- **Farebnné odlíšenie** - Rôzne farby pre lepšiu vizuálnu orientáciu
- **Formátované veľkosti** - Automatický prepočet na B, KB, MB, GB, TB
- **Real-time info** - Zobrazenie aktuálneho priečinka a celkovej veľkosti

## Kompilácia

```bash
cargo build --release
```

## Spustenie

```bash
./target/release/disk-analyzer
```

Alebo cez cargo:

```bash
cargo run --release
```

## Použitie

1. **Zadajte cestu** - Do poľa "Cesta" zadajte absolútnu cestu k priečinku, ktorý chcete analyzovať
   - Predvolená je domovský priečinok (`$HOME`)

2. **Kliknite na "📂 Skenovať"** - Spustí rekurzívne skenovanie celého priečinka a všetkých podpriečinkov

3. **Preskúmajte výsledky**:
   - **Jeden klik** na blok - zobrazí detaily (názov, veľkosť, cesta)
   - **Dvojklik** na priečinok - vstúpite dovnútra a uvidíte jeho obsah
   - **⬅ Späť** - návrat do predchádzajúceho priečinka
   - **🏠 Koreň** - návrat na začiatok (koreňový priečinok)

4. **Navigujte hierarchiou** - Postupne sa vŕtajte do najväčších priečinkov a hľadajte, čo zabera miesto

## Príklady ciest na analýzu

- `/home/miro` - Domovský priečinok
- `/var/log` - Systémové logy
- `/usr` - Systémové aplikácie
- `/home/miro/Downloads` - Stiahnuté súbory

## Technické detaily

- **GUI framework**: egui/eframe
- **Skenovanie súborov**: walkdir
- **Vizualizácia**: Vlastná implementácia treemap algoritmu
- **Jazyk**: Rust (edition 2021)

## Poznámky

- Aplikácia vyžaduje oprávnenia na čítanie analyzovaných priečinkov
- Pri analýze veľkých priečinkov môže skenovanie trvať dlhšie (skenuje sa všetko rekurzívne)
- Symbolické odkazy nie sú sledované, aby sa predišlo zacykleniu
- Aplikácia správne počíta veľkosti priečinkov ako súčet všetkých súborov v nich
- Skenovanie prebieha v samostatnom vlákne, GUI zostáva responzívne
