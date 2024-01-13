from pathlib import Path
from pydantic import Field, computed_field
from pydantic_settings import BaseSettings

class EnvCfg(BaseSettings):
    RUNS_DIR: Path = Field(default=Path("/usr/src/ultralytics/runs"),
                           description="Output of ultralytics training and prediction runs")
    
    PYTHON_SRC_DIR: Path = Field(default=Path("/usr/src/python"),
                                    description="Directory containing python source code")

    DETECT_MODEL_NAME: str = Field(default="yolo"
                                      , description="Name of the model used for detection, used to name model in runs dir")
    
    CLASSIFY_MODEL_NAME: str = Field(default="classifier")

    TRAIN_FOLDER_NAME: str = "train"
    VALID_FOLDER_NAME: str = "valid"
    TEST_FOLDER_NAME: str = "test"
    LABEL_FOLDER_NAME: str = "labels"
    IMAGE_FOLDER_NAME: str = "images"

    DETECT_IMG_SZ: int = 640
    CLASSIFY_IMG_SZ: int = 128

    @computed_field
    @property
    def INCOMING_PATH(self) -> Path:
        return self.PYTHON_SRC_DIR / "datasets/incoming"

    @computed_field
    @property
    def PYTORCH_CNN_MODEL_PATH(self) -> Path:
        return self.PYTHON_SRC_DIR / "card_cnn.pt"

    @computed_field
    @property
    def CARD_YOLO_PATH(self) -> Path:
        # contains label studio YOLO export 
        # this is where the yolo export from label studio was unzipped, it should contain an images and labels folder
        return self.PYTHON_SRC_DIR / "datasets/all3"
    
    @computed_field
    @property
    def LIVE_PATH(self) -> Path:
        # Live screenshots & guessed predictions
        # In Yolo format
        return self.PYTHON_SRC_DIR / "datasets/live"
    
    @computed_field
    @property
    def LIVE_CARD_IMAGES_PATH(self) -> Path:
        return self.PYTHON_SRC_DIR / "datasets/live_images"
    
    @computed_field
    @property
    def DETECT_DATA_PATH(self) -> Path:
        # collapsed into single card class and split into train and validate
        return self.PYTHON_SRC_DIR / "datasets" / "zynga"

    # Base dataset directory for classification, based on CARD_YOLO_PATH
    @computed_field
    @property
    def CLASSIFY_DATA_PATH(self) -> Path:
        return self.PYTHON_SRC_DIR / "datasets/card_cnn"  
    
    @computed_field
    @property
    def CLASSIFY_PROJECT_PATH(self) -> Path:
        return self.RUNS_DIR / 'classify'

