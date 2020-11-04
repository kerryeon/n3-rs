from n3 import *


class DummyImageClassification(Trainer):
    def train(self, handler):
        # Step 1. ready to train
        self.model.train()
        self.optimizer._initialize(self.model)

        data = iter(self.data.get_train_dataset())
        _x, _classes = next(data)

    def eval(self, handler):
        raise NotImplementedError
