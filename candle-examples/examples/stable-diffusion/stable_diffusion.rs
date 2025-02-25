use crate::schedulers::PredictionType;
use crate::{clip, ddim, unet_2d, vae};
use candle::{DType, Device, Result};
use candle_nn as nn;

#[derive(Clone, Debug)]
pub struct StableDiffusionConfig {
    pub width: usize,
    pub height: usize,
    pub clip: clip::Config,
    pub clip2: Option<clip::Config>,
    autoencoder: vae::AutoEncoderKLConfig,
    unet: unet_2d::UNet2DConditionModelConfig,
    scheduler: ddim::DDIMSchedulerConfig,
}

impl StableDiffusionConfig {
    pub fn v1_5(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        let bc = |out_channels, use_cross_attn, attention_head_dim| unet_2d::BlockConfig {
            out_channels,
            use_cross_attn,
            attention_head_dim,
        };
        // https://huggingface.co/runwayml/stable-diffusion-v1-5/blob/main/unet/config.json
        let unet = unet_2d::UNet2DConditionModelConfig {
            blocks: vec![
                bc(320, Some(1), 8),
                bc(640, Some(1), 8),
                bc(1280, Some(1), 8),
                bc(1280, None, 8),
            ],
            center_input_sample: false,
            cross_attention_dim: 768,
            downsample_padding: 1,
            flip_sin_to_cos: true,
            freq_shift: 0.,
            layers_per_block: 2,
            mid_block_scale_factor: 1.,
            norm_eps: 1e-5,
            norm_num_groups: 32,
            sliced_attention_size,
            use_linear_projection: false,
        };
        let autoencoder = vae::AutoEncoderKLConfig {
            block_out_channels: vec![128, 256, 512, 512],
            layers_per_block: 2,
            latent_channels: 4,
            norm_num_groups: 32,
        };
        let height = if let Some(height) = height {
            assert_eq!(height % 8, 0, "height has to be divisible by 8");
            height
        } else {
            512
        };

        let width = if let Some(width) = width {
            assert_eq!(width % 8, 0, "width has to be divisible by 8");
            width
        } else {
            512
        };

        Self {
            width,
            height,
            clip: clip::Config::v1_5(),
            clip2: None,
            autoencoder,
            scheduler: Default::default(),
            unet,
        }
    }

    fn v2_1_(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
        prediction_type: PredictionType,
    ) -> Self {
        let bc = |out_channels, use_cross_attn, attention_head_dim| unet_2d::BlockConfig {
            out_channels,
            use_cross_attn,
            attention_head_dim,
        };
        // https://huggingface.co/stabilityai/stable-diffusion-2-1/blob/main/unet/config.json
        let unet = unet_2d::UNet2DConditionModelConfig {
            blocks: vec![
                bc(320, Some(1), 5),
                bc(640, Some(1), 10),
                bc(1280, Some(1), 20),
                bc(1280, None, 20),
            ],
            center_input_sample: false,
            cross_attention_dim: 1024,
            downsample_padding: 1,
            flip_sin_to_cos: true,
            freq_shift: 0.,
            layers_per_block: 2,
            mid_block_scale_factor: 1.,
            norm_eps: 1e-5,
            norm_num_groups: 32,
            sliced_attention_size,
            use_linear_projection: true,
        };
        // https://huggingface.co/stabilityai/stable-diffusion-2-1/blob/main/vae/config.json
        let autoencoder = vae::AutoEncoderKLConfig {
            block_out_channels: vec![128, 256, 512, 512],
            layers_per_block: 2,
            latent_channels: 4,
            norm_num_groups: 32,
        };
        let scheduler = ddim::DDIMSchedulerConfig {
            prediction_type,
            ..Default::default()
        };

        let height = if let Some(height) = height {
            assert_eq!(height % 8, 0, "height has to be divisible by 8");
            height
        } else {
            768
        };

        let width = if let Some(width) = width {
            assert_eq!(width % 8, 0, "width has to be divisible by 8");
            width
        } else {
            768
        };

        Self {
            width,
            height,
            clip: clip::Config::v2_1(),
            clip2: None,
            autoencoder,
            scheduler,
            unet,
        }
    }

    pub fn v2_1(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        // https://huggingface.co/stabilityai/stable-diffusion-2-1/blob/main/scheduler/scheduler_config.json
        Self::v2_1_(
            sliced_attention_size,
            height,
            width,
            PredictionType::VPrediction,
        )
    }

    fn sdxl_(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
        prediction_type: PredictionType,
    ) -> Self {
        let bc = |out_channels, use_cross_attn, attention_head_dim| unet_2d::BlockConfig {
            out_channels,
            use_cross_attn,
            attention_head_dim,
        };
        // https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/main/unet/config.json
        let unet = unet_2d::UNet2DConditionModelConfig {
            blocks: vec![
                bc(320, None, 5),
                bc(640, Some(2), 10),
                bc(1280, Some(10), 20),
            ],
            center_input_sample: false,
            cross_attention_dim: 2048,
            downsample_padding: 1,
            flip_sin_to_cos: true,
            freq_shift: 0.,
            layers_per_block: 2,
            mid_block_scale_factor: 1.,
            norm_eps: 1e-5,
            norm_num_groups: 32,
            sliced_attention_size,
            use_linear_projection: true,
        };
        // https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/main/vae/config.json
        let autoencoder = vae::AutoEncoderKLConfig {
            block_out_channels: vec![128, 256, 512, 512],
            layers_per_block: 2,
            latent_channels: 4,
            norm_num_groups: 32,
        };
        let scheduler = ddim::DDIMSchedulerConfig {
            prediction_type,
            ..Default::default()
        };

        let height = if let Some(height) = height {
            assert_eq!(height % 8, 0, "height has to be divisible by 8");
            height
        } else {
            1024
        };

        let width = if let Some(width) = width {
            assert_eq!(width % 8, 0, "width has to be divisible by 8");
            width
        } else {
            1024
        };

        Self {
            width,
            height,
            clip: clip::Config::sdxl(),
            clip2: Some(clip::Config::sdxl2()),
            autoencoder,
            scheduler,
            unet,
        }
    }

    pub fn sdxl(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        Self::sdxl_(
            sliced_attention_size,
            height,
            width,
            // https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/main/scheduler/scheduler_config.json
            PredictionType::Epsilon,
        )
    }

    pub fn build_vae<P: AsRef<std::path::Path>>(
        &self,
        vae_weights: P,
        device: &Device,
        dtype: DType,
    ) -> Result<vae::AutoEncoderKL> {
        let weights = unsafe { candle::safetensors::MmapedFile::new(vae_weights)? };
        let weights = weights.deserialize()?;
        let vs_ae = nn::VarBuilder::from_safetensors(vec![weights], dtype, device);
        // https://huggingface.co/runwayml/stable-diffusion-v1-5/blob/main/vae/config.json
        let autoencoder = vae::AutoEncoderKL::new(vs_ae, 3, 3, self.autoencoder.clone())?;
        Ok(autoencoder)
    }

    pub fn build_unet<P: AsRef<std::path::Path>>(
        &self,
        unet_weights: P,
        device: &Device,
        in_channels: usize,
        use_flash_attn: bool,
        dtype: DType,
    ) -> Result<unet_2d::UNet2DConditionModel> {
        let weights = unsafe { candle::safetensors::MmapedFile::new(unet_weights)? };
        let weights = weights.deserialize()?;
        let vs_unet = nn::VarBuilder::from_safetensors(vec![weights], dtype, device);
        let unet = unet_2d::UNet2DConditionModel::new(
            vs_unet,
            in_channels,
            4,
            use_flash_attn,
            self.unet.clone(),
        )?;
        Ok(unet)
    }

    pub fn build_scheduler(&self, n_steps: usize) -> Result<ddim::DDIMScheduler> {
        ddim::DDIMScheduler::new(n_steps, self.scheduler)
    }
}

pub fn build_clip_transformer<P: AsRef<std::path::Path>>(
    clip: &clip::Config,
    clip_weights: P,
    device: &Device,
    dtype: DType,
) -> Result<clip::ClipTextTransformer> {
    let weights = unsafe { candle::safetensors::MmapedFile::new(clip_weights)? };
    let weights = weights.deserialize()?;
    let vs = nn::VarBuilder::from_safetensors(vec![weights], dtype, device);
    let text_model = clip::ClipTextTransformer::new(vs, clip)?;
    Ok(text_model)
}
