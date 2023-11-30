# replicate-rs
An un-official minimal async client for [Replicate](https://replicate.com/).   
Provides simple type safe functionality for interacting with Replicate models with [serde](https://serde.rs/) and [isahc](https://docs.rs/isahc/latest/isahc/).

<a href="https://crates.io/crates/replicate-rs"><img src="https://img.shields.io/crates/v/replicate-rs"></a>

## API Coverage

#### Predictions
- [x] [Create a Prediction](https://replicate.com/docs/reference/http#predictions.create)
- [x] [Get a Prediction](https://replicate.com/docs/reference/http#predictions.get)
- [x] [List Predictions](https://replicate.com/docs/reference/http#predictions.list)
- [x] [Cancel a Prediction](https://replicate.com/docs/reference/http#predictions.cancel)

#### Models
- [ ] [Create a Model](https://replicate.com/docs/reference/http#models.create)
- [x] [Get a Model](https://replicate.com/docs/reference/http#models.get)
- [x] [Get a Model Version](https://replicate.com/docs/reference/http#models.versions.get)
- [x] [List a Model's Versions](https://replicate.com/docs/reference/http#models.versions.list)
- [ ] [Delete a Model Version](https://replicate.com/docs/reference/http#models.versions.delete)
- [x] [List Public Models](https://replicate.com/docs/reference/http#models.list)

#### Collections
- [ ] [Get a Collection of Models](https://replicate.com/docs/reference/http#collections.get)
- [ ] [List Collection of Models](https://replicate.com/docs/reference/http#collections.list)

#### Hardware
- [ ] [List available hardware for a Model](https://replicate.com/docs/reference/http#hardware.list)

#### Training
- [ ] [Create a Training](https://replicate.com/docs/reference/http#trainings.create)
- [ ] [Get a Training](https://replicate.com/docs/reference/http#trainings.get)
- [ ] [List Trainings](https://replicate.com/docs/reference/http#trainings.list)
- [ ] [Cancel a Training](https://replicate.com/docs/reference/http#trainings.cancel)
