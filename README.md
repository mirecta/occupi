# Disk Analyzer - Analýza obsadenosti úložiska

Desktopová aplikácia v Ruste na vizualizáciu obsadenosti diskového priestoru pomocou treemap grafov (štvorcové grafy).

## Vlastnosti

- **Treemap vizualizácia** - Štvorcové grafy kde veľkosť bloku reprezentuje veľkosť súboru/priečinka
- **Interaktívne** - Kliknutie na blok zobrazí detaily (názov, veľkosť, cesta)
- **Nastaviteľná hĺbka** - Možnosť nastaviť do akej úrovne adresárového stromu skenovať (1-5)
- **Farebnné odlíšenie** - Rôzne farby pre lepšiu vizuálnu orientáciu
- **Formátované veľkosti** - Automatický prepočet na B, KB, MB, GB, TB

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

2. **Nastavte hĺbku** - Posuvník "Hĺbka" určuje koľko úrovní podpriečinkov sa má skenovať
   - Nižšia hodnota = rýchlejšie skenovanie
   - Vyššia hodnota = podrobnejšia analýza

3. **Kliknite na "Skenovať"** - Spustí skenovanie vybraného priečinka

4. **Preskúmajte výsledky** - Kliknite na jednotlivé bloky pre zobrazenie detailov

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
- Pri analýze veľkých priečinkov môže skenovanie trvať dlhšie
- Symbolické odkazy nie sú sledované (follow_links = false)
