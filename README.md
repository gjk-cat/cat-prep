# Cat-Prep - mdBook preprocesor na tvorbu úložiště studijních materiálů
Tento preprocesor umožňuje generování hierarchie učitelů,
předmětů, materiálů a tagů uvnitř `mdBook` knihy.

Přestože nejlépe funguje s `html` renderererm,
nemá implementovaná omezení na backend.
Výstup v LaTeXovém a EPUB backendu je momentálně
méně než potěšující, chyba však stojí na straně rendererů
- nemají kompletní podporu markdownu a HTML.

`cat-prep` je tedy future-proof a jeho výstup v těchto
formátech se bude zlepšovat s tím, jak se budou zlepšovat
dané renderery.

## Požadavky
- libovolný Linux
- následující programy:
	- `ls`
	- `xargs`
	- `git`
	- `sh`
	- `true`
	- `mdbook` a jeho `links` preprocesor (musí být v `book.toml` specifikován před `cat-prepem`)
- aby `cat-prep` mohl pracovat, musí `mdbook` běžet uvnitř gitového repozitáře


## Dokumentace

K API dokumentaci je možné přistoupit následujícími způsoby
- `cargo doc --no-deps --open` (`--no-deps` slouží k vynechání závislostí, pro rychlejší kompilaci dokumentace)i
- <https://docs.rs/mdbook-cat-prep/>

Většina symbolů je má krátkou a dlouhou dokumentaci,
vyplatí se je tedy rozkliknout.

## Instalace

Nejjednodušším způsobem instalace je instalace pomocí nástroje `cargo`.

Pro instalaci poslední publikované verze:
```sh
cargo install mdbook-cat-prep

```

Pro instalaci nejnovější verze z gitu:

```sh
cargo install --git "https://github.com/gjk-cat/cat-prep"

```

Dále je možná manuální instalace uvnitř naklonovaného repozitáře:

```sh
git clone https://github.com/gjk-cat/cat-prep
cd cat-prep
cargo install --path .
```

Pozn. instalace v debug módu umožňuje nahlížet na CatContext.

## Použití

Předpokladem použití `cat-prepu` je splnění požadavků vypsaných výše
a přítomnost programu `mdbook-cat-prep` v `PATH` ať už prostřednictvím instalace
nebo jiným způsobem.

0. Nejdříve je potřeba v `mdbooku` nastavit, že má používat preprocessor `cat-prep`

```toml
# v souboru book.toml na konec
[preprocessor.cat-prep]
```

další doporučená nastavení:

```toml
[output.html]
theme = "src/theme" # kde src/theme vede k naklonovanému kočičkovému tématu.
# pro jeho správnou funkcí je zapotřebí kočičkový obrázek, který je k nalezení
# ve většině repozitářů. Popř. jde nahradit libovolným jiným obrázkem.
# Zvířátka preferována
```

1. Poté je zapotřebí vytvořit složku `teachers` a, pokud možno,
prvního učitele pro váš účet.

```sh
mkdir teachers
kak teachers/jmeno.toml # zvolte libovolný editor
```

V souboru `jmeno.toml`:

```toml
jmeno = "Lukáš Hozda"         # má odpovídat jménu, které osoba používá v gitu, tj. `user.name`
email = "luk.hozda@gmail.com" # odpovídá email, který osoba používá v gitu, tj. `user.email`
username = "magnusi"          # pro kontexty, kde se nevyplatí používat email nebo jméno, např. odkazy
# libovolný popisek, formátován jako markdown, může sloužit na extra informace
# doporučuje se začínat na třetí úrovni nadpisů v zájmu přehlednosti seznamu vyučujících
bio = """
Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Sed lacinia hendrerit placerat.

### Mauris posuere libero dui
non feugiat nunc imperdiet non. Ut pharetra sodales mi,
quis __sagittis__ velit __tristique__ tincidunt.[1]

### Sed in tellus tincidunt, molestie libero vel,
semper mi. Praesent lacus felis, `aliquam` in tempor vel,
fringilla eget dui. Quisque *tristique* pulvinar fringilla.

[1]: <https://www.lipsum.com/feed/html>
"""
```

2. Následně je možné vytvořit libovolné stránky, které nejsou součástí předmětů,
viz <https://rust-lang.github.io/mdBook/>.

3. Tvorba prvního předmětu. Umístění předmětu může být libovolné, pokud splňuje tyto tři podmínky:
	1. je ve podsložce složky `src` (teoreticky, v `src` by to fungovalo také, ale potom by mohl existovat jen jeden předmět)
	2. není v podsložce jiného předmětu
	3. složka ve které je obsahuje soubor `subject.md`

```sh
mkdir -p src/predmety/predmet1
touch src/predmety/predmet1/subject.md
kak src/predmety/predmet1/subject.md # nebo libovolný jiný editor
```

Následně je v souboru `subject.md` vytvořit header a nějaký počáteční obsah.

```markdown
nazev = "Můj první předmět"
zodpovedna_osoba = "Lukáš Hozda"
# ↑ pokud možno, mělo by odpovídat jménu, emailu nebo usernamu některého vyučujícícho
#   pokud je autorem někdo jiný, zadejte email.
bio = "krátký popisek předmětu"

+++

## Můj první předmět

Lorem ipsum dolor sit amet, consectetur adipiscing elit.
In varius lacinia risus eu vehicula. Vestibulum consectetur
feugiat dignissim. Mauris sed leo id lectus commodo egestas.
Integer sed ligula quis lorem viverra fringilla lobortis at elit.
Fusce a eros laoreet, dictum enim et, pellentesque erat.

```

Na konci souboru by nemělo být nic, co by mohlo narušit výpis jednotlivých materiálů v předmětu.

4. Dále je možné vytvořit první materiál tohoto předmětu

```sh
touch src/predmety/predmet1/material1.md
kak src/predmety/predmet1/material1.md # nebo libovolný editor

```

Následně je zapotřebí vytvořit v souboru `material.md` hlavičku a počáteční obsah.

```markdown
nazev = "Můj první článek"
tagy = ["tag1", "tag2", "tag3"]
datum = "20.1.2019"  # dobrovolné a libovolné. Jelikož "datum" je singulár  od "data", lze použít jakkoliv :^)

+++

# Můj první článek

Lorem ipsum dolor sit amet, consectetur adipiscing elit.
In varius lacinia risus eu vehicula. Vestibulum consectetur
feugiat dignissim. Mauris sed leo id lectus commodo egestas.
Integer sed ligula quis lorem viverra fringilla lobortis at elit.
Fusce a eros laoreet, dictum enim et, pellentesque erat.
```

5. Nyní nesmíme zapomenout zmínit tyto soubory v `SUMMARY.md` (viz návod u `mdbook`),
jinak je `mdbook` a tudíž ani `cat-prep` neuvidí.
Kočičková hierarchie je paralelní hierarchii `SUMMARY.md`, ale není špatný nápad
reflektovat kočičkovou hierarchii v té kočičkové.

```markdown
...

- [Předmět 1](./predmety/predmet1/subject.md)
	- [Článek 1](./predmety/predmet1/material1.md)

...
```

6. Nyní je možné knihu sestavit:

```
mdbook build
```

nebo spustit live-reload server pro další vývoj knihy:

```
mdbook serve
```

### Šablona
Alternativně lze využít šablonu z repozitáře <https://github.com/gjk-cat/sablona>

## License

Soubor je licencován open-source licencí Fair:

```
Lukáš Hozda <me@magnusi.tech> 2020 (c)

Usage of the works is permitted provided that this instrument is retained with the works, so that any entity that uses the works is notified of this instrument.

DISCLAIMER: THE WORKS ARE WITHOUT WARRANTY.
```


