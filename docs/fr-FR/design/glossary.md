<a name="xdzip-xdzip-terminology-and-implementation-limits"></a>

# Terminologie et limites d’implémentation de XDZip

[English](../../design/glossary.md) | [简体中文](../../zh-CN/design/glossary.md) | [繁體中文](../../zh-TW/design/glossary.md) | [日本語](../../ja-JP/design/glossary.md) | [한국어](../../ko-KR/design/glossary.md) | Français | [Deutsch](../../de-DE/design/glossary.md) | [Español](../../es-ES/design/glossary.md) | [Português (Brasil)](../../pt-BR/design/glossary.md) | [Русский](../../ru-RU/design/glossary.md)

<a name="xdzip-1-terminology"></a>

## 1. Terminologie

| Terme | Définition |
| --- | --- |
| Archive | Conteneur ou flux compressé pris en charge, avec sa source autorisée. |
| Codec | Implémentation qui détecte, analyse, lit ou crée un format. |
| Session | Source ouverte, moteur, index et contexte de navigation. Les tâches ont un cycle distinct. |
| Entrée | Enregistrement réel de l’archive. Un dossier peut également avoir un nœud d’affichage synthétique. |
| SourceVersion | Empreinte calculée au mieux pour un fichier ou volume source admis. |
| QuerySnapshot | Ordre immuable des résultats actuels, indexé par session, requête, génération et révision. |
| Génération | Remplacement des requêtes de l’interface pour empêcher un ancien résultat d’écraser le plus récent. |
| Opération préparée | Entrée d’exécution bornée, avec la source, la sélection et les options utiles à l’opération. |
| Tâche | Cycle de vie et résultat d’une opération admise dans la file de tâches. |
| Autorisation | Droit d’accès à un fichier ou un dossier détenu par l’hôte, distinct d’un chemin. |
| Sortie | Interface d’écriture soumise au comptage, aux quotas et au contrôle transactionnel. |
| Processus ou fil de travail | Les commandes et tâches utilisent des fils de l’hôte. Les travailleurs d’archives et d’images sont des processus séparés. |
| Validation finale | Publication de la sortie temporaire vérifiée par l’opération autorisée sur le fichier ou l’archive. |
| Instantané de tâche | Dernier rapport d’état, de progression, de demande ou de résultat. Des instantanés intermédiaires peuvent être regroupés. |
| Capacités | Intersection de l’implémentation, des fonctions du format, de la plateforme et de la politique. |
| Compression solide | Des fichiers partagent des blocs compressés. La lecture sélective peut nécessiter de décoder ce qui précède. |
| Première image présentée | Image réellement rendue et présentée. Un journal ui_ready ne la remplace pas. |

<a name="xdzip-2-units-and-values"></a>

## 2. Unités et valeurs

La mémoire et les fichiers se mesurent en unités binaires KiB/MiB/GiB, avec 1 MiB = 1024² octets. Le transport et le décompte conservent des nombres entiers d’octets. Le temps s’exprime en ms ou s. Les durées d’exécution utilisent une horloge monotone. Les dates de fichiers conservent la représentation calendaire et la précision de la source lorsqu’elles sont disponibles. Le débit est exprimé en MiB/s en précisant s’il concerne l’entrée compressée, les données décodées ou la sortie.

Un CPU à 100% d’un cœur occupe entièrement un cœur logique. Un arbre de processus peut dépasser 100%. PSS, RSS, USS et empreinte physique désignent des mesures distinctes. La taille du téléchargement compressé, la taille installée et celle de l’exécutable sont enregistrées séparément.

<a name="xdzip-3-where-to-find-current-limits"></a>

## 3. Où trouver les limites actuelles

| Sujet | Référence d’implémentation |
| --- | --- |
| Requêtes, pages et limites de la boîte de messages | [Passerelle Qt/Rust](qt-rust-bridge.md) |
| Admission des tâches, progression et codes d’état | [Ordonnancement](task-scheduling.md) |
| Limites d’extraction, autorisation et publication | [Sécurité des fichiers](file-security.md) |
| Limites des métadonnées et du ZIP indexé sur disque | [Indexation des grandes archives](../engineering/large-index.md) |
| Limites des sources et volumes | [Ouverture des archives](../engineering/archive-opening.md) |
| Limites des aperçus de texte et d’images | [Aperçu du contenu](../engineering/content-preview.md) |
| Trames, supervision et ressources des processus de format | [Processus d’archives](../engineering/native-codecs.md) |
| Protocole d’extension expérimental | [Essai du SDK](../engineering/sdk-trial.md) |
| Objectifs de performance | [Conception des performances](performance.md) |

<a name="xdzip-4-versioning"></a>

## 4. Versions

La version de l’application, l’interface de construction interne CXX, le protocole externe et le schéma des données persistantes ont des versions distinctes. La version du produit seule ne détermine pas celle du protocole SDK. Voir la [conception du SDK de formats](codec-extension-sdk.md).
